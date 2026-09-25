use std::{io::Write, net::{IpAddr, Ipv4Addr}, str::FromStr};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::Receiver, sync::oneshot};
use tokio_util::{codec::{Framed, LinesCodec}};
use futures::{StreamExt, SinkExt};
use std::error::Error;
use std::collections::HashMap;

// does this serve any purpose?
const PORT: u16 = 6767;
const LISTENER_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

pub struct Command {
    pub show: bool,
    pub respond_to: oneshot::Sender<()>,
}

pub struct Peers
{
    pub peer_map: HashMap<String, Ipv4Addr>,
    pub channel: Option<(Ipv4Addr, String)>,
}

#[derive(Debug)]
pub struct Inbox
{
    pub inbox: Vec<(IpAddr, String)>,
}

pub fn show_inbox(inbox: &mut Inbox)
{
    let mut it = inbox.inbox.iter();
    while let Some((addr, msg)) = it.next()
    {
        println!("{addr}: {msg}");
    }
}

/*
* Letting OS magic handle concurrency instead of spawning
* a new thread per connection request since the connections 
* last for a very short duration
*/
pub async fn listen(rx: &mut Receiver<Command>, inbox: &mut Inbox) -> Result<(), Box<dyn Error>>
{
    let stream = TcpListener::bind((LISTENER_ADDR, PORT)).await?;
    
    loop {
        tokio::select! {
            accept_res = stream.accept() => {
                match accept_res {
                    Ok((socket, addr)) => {
                        let mut framed = Framed::new(socket, LinesCodec::new());
                        
                        while let Some(result) = framed.next().await {
                            match result {
                                Ok(line) => {
                                    inbox.inbox.push((addr.ip(), line));
                                }
                                Err(_) => {},
                            }
                        }
                    },
                    Err(_) => println!("\nError accepting connection"),
                }
            }

            Some(msg) = rx.recv() => {
                if msg.show 
                {
                    show_inbox(inbox);
                }
                let _ = msg.respond_to.send(());
            }
        }
    }
}

pub async fn send(dest: Ipv4Addr, msg: String) -> Result<(), Box<dyn Error>>
{
    let res_stream = TcpStream::connect((dest, PORT)).await;
    match res_stream {
        Ok(stream) => {
            let mut framed_conn = Framed::new(stream, LinesCodec::new());
            let res = framed_conn.send(msg).await;
            match res {
                Ok(()) => {},
                Err(_) => {println!("Failed to send message");},
            }
        },
        Err(_) => println!("Failed to connect to client (User might be offline)"),
    }
    Ok(())
}

pub async fn process_command(peers: &mut Peers, peer_nick: &String) -> Result<(bool, bool), Box<dyn Error>>
{
    print!("[{peer_nick}]> ");
    std::io::stdout().flush()?;
    
    let mut buf = String::new();
    // PONDER: Possibility of injection here?
    let _ = std::io::stdin().read_line(&mut buf)?;
    
    let args = buf.split_once(" ");
    let mut tx_flag = false;
    match args {
        // TODO: Add list channels command
        Some((cmd, arg)) => {
            let arg = arg.trim();
            match cmd {
                "add" => {
                    if let Some((peer_nick, addr_str)) = arg.split_once(" ")
                    {
                        let parsed_addr = Ipv4Addr::from_str(addr_str);
                        match parsed_addr {
                            Ok(addr) => { 
                                let _ = &peers.peer_map.insert(String::from(peer_nick), addr); 
                            },
                            Err(e) => { eprintln!("{e}"); },
                        }
                    }
                    else {
                        println!("Invalid command. Use help to see usage");
                    }
                }
                "send" => { 
                    match &peers.channel {
                        Some(ch) => {
                            send(ch.0, String::from(arg)).await?;
                        },
                        None => println!("No Channel selected: Use channel <name_of_user> to set a channel"),
                    }
                },
                "channel" => {
                    // add more safe parsing here
                    peers.channel = Some((peers.peer_map[arg], String::from(arg)));
                },
                "close" => {
                    if arg == "channel" {
                        peers.channel = None;
                    }
                    else {
                        println!("Invalid Command: Use help");
                    }
                },
                _ => println!("Invalid Command: Use help"),
            }
        },
        None => {
            let cmd = buf.trim();
            
            match cmd {
                "inbox" => tx_flag = true,
                "exit" => return Ok((true, tx_flag)),
                "help" => {
                    println!("Commands:\n");
                    println!("send: Send message to selected channel. Usage: send <msg>");
                    println!("channel: Set channel to send messages. Usage: channel <username>");
                    println!("add: Add new channel to address book. Usage: add <username> <ipv4addr>");
                    println!("close channel: Close current channel");
                    println!("exit: Close program");
                },
                _ => println!("Invalid Command: Use help"),
            }
        }
    }
    
    Ok((false, tx_flag))
}

