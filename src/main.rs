use std::net::Ipv4Addr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::{codec::{Framed, LinesCodec}};
use futures::{StreamExt, SinkExt};
use std::error::Error;
use std::collections::HashMap;

const PORT: u16 = 6767;
const LISTENER_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

pub struct Peers
{
    peer_map: HashMap<String, Ipv4Addr>,
    channel: Option<Ipv4Addr>,
}

impl Peers
{
    
    pub async fn process_command(&mut self, cli_input: &mut String) -> Result<bool, Box<dyn Error>>
    {
        let args = cli_input.split_once(" ");
        
        if let Some((cmd, arg)) = args
        {
            let arg = arg.trim();
            
            match cmd 
            {
                "add" => {
                    // split arg into => user and ipaddr
                }
                "send" => { 
                    match self.channel {
                        Some(ch) => self.send_msg(ch, String::from(arg)).await?,
                        None => println!("No Channel selected: Use channel <name_of_user> to set a channel"),
                    }
                },
                "channel" => {
                    self.channel = Some(self.peer_map[arg]);
                },
                "close" => {
                    if arg == "channel" {
                        self.channel = None;
                    }
                    else {
                        println!("Invalid Command: Use --help");
                    }
                }
                "exit" => return Ok(true),
                "--help" => {
                    println!("Commands:");
                    println!("send: Send message to selected channel. Usage: send <msg>");
                    println!("channel: Set channel(peer) to send messages. Usage: channel <username>");
                    // TODO: Continue this later
                },
                _ => println!("Invalid Command: Use --help"),
            }
        }
        
        Ok(false)
    }
    
    pub async fn send_msg(&self, dest: Ipv4Addr, msg: String) -> Result<(), Box<dyn Error>>
    {
        let stream = TcpStream::connect((dest, PORT)).await?;
        let mut framed_conn = Framed::new(stream, LinesCodec::new());

        framed_conn.send(msg).await?;
        Ok(())
    }
    
    /*
     * Im assuming this moves any accepted connection 
     * to a non blocking thread and continues listening
     */
    pub async fn listen_for_msg(&self) -> Result<(), Box<dyn Error>>
    {
        let stream = TcpListener::bind((LISTENER_ADDR, PORT)).await?;
        
        loop {
            let (socket, addr) = stream.accept().await?;
            
            tokio::spawn(async move {
                let mut framed = Framed::new(socket, LinesCodec::new());
                
                while let Some(result) = framed.next().await {
                    match result {
                        Ok(line) => {
                            println!("{addr}: {line}");
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            break;
                        }
                    }
                }
                println!("Connection closed.");
            });
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    // let args: Vec<String> = std::env::args().collect();
    // let mut peer_arg_iter =  args.iter();
    // let _ =  peer_arg_iter.next().unwrap();
    // let peer_arg =  peer_arg_iter.next().unwrap();
    
    let mut peers = Peers {
        peer_map: HashMap::new(),
        channel: None,
    };

    let mut exit_flag = false;
    
    while !exit_flag {
        let mut buf = String::new();
        
        // PONDER: Possibility of injection here?
        let _ = std::io::stdin().read_line(&mut buf)?;
        exit_flag = peers.process_command(&mut buf).await?;
    }
    
    // match peer_arg.as_str() {
    //     "PEER1" => {
    //         println!("PEER1");
    //         peers.send_msg(PEER2, String::from("hallo PEER2")).await?;
    //     },
    //     "PEER2" => {
    //         println!("PEER2");
    //         peers.listen_for_msg().await?;
    //     },
    //     _ => {}
    // }
    Ok(())
}
