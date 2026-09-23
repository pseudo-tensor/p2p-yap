use std::{net::Ipv4Addr, str::FromStr};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::{codec::{Framed, LinesCodec}};
use futures::{StreamExt, SinkExt};
use std::error::Error;
use std::collections::HashMap;

// does this serve any purpose?
const PORT: u16 = 6767;
const LISTENER_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

// TODO: inbox command to put incoming messages
// instead of spillin everything in stdout
pub struct Peers
{
    pub peer_map: HashMap<String, Ipv4Addr>,
    pub channel: Option<Ipv4Addr>,
}

pub async fn process_command(peers: &mut Peers, cli_input: &mut String) -> Result<bool, Box<dyn Error>>
{
    let args = cli_input.split_once(" ");
    match args {
        Some((cmd, arg)) => {
            let arg = arg.trim();
            match cmd {
                "add" => {
                    if let Some((peer_nick, addr_str)) = arg.split_once(" ")
                    {
                        let parsed_addr = Ipv4Addr::from_str(addr_str);
                        match parsed_addr {
                            Ok(addr) => { peers.peer_map.insert(String::from(peer_nick), addr); },
                            Err(e) => { eprintln!("{e}"); },
                        }
                    }
                    else {
                        println!("Invalid command. Use help to see usage");
                    }
                }
                "send" => { 
                    match peers.channel {
                        Some(ch) => send(ch, String::from(arg)).await?,
                        None => println!("No Channel selected: Use channel <name_of_user> to set a channel"),
                    }
                },
                "channel" => {
                    peers.channel = Some(peers.peer_map[arg]);
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
            let cmd = cli_input.trim();
            
            match cmd {
                "exit" => return Ok(true),
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
    
    Ok(false)
}

pub async fn send(dest: Ipv4Addr, msg: String) -> Result<(), Box<dyn Error>>
{
    let stream = TcpStream::connect((dest, PORT)).await?;
    let mut framed_conn = Framed::new(stream, LinesCodec::new());

    framed_conn.send(msg).await?;
    Ok(())
}

/*
* Letting OS magic handle concurrency instead of spawning
* a new thread per connection request since the connections 
* last for a very short duration
*/
pub async fn listen() -> Result<(), Box<dyn Error>>
{
    let stream = TcpListener::bind((LISTENER_ADDR, PORT)).await?;
    
    loop {
        let (socket, addr) = stream.accept().await?;
        let mut framed = Framed::new(socket, LinesCodec::new());
        
        while let Some(result) = framed.next().await {
            match result {
                Ok(line) => {
                    // Print user nick instead of addr here
                    println!("{addr}: {line}");
                }
                Err(e) => {
                    eprintln!("{e}");
                    break;
                }
            }
        }
    }
}