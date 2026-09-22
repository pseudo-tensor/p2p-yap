use std::net::Ipv4Addr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::{codec::{Framed, LinesCodec}};
use futures::{StreamExt, SinkExt};
use std::error::Error;

const PORT: u16 = 8080;
// listener
const PEER2: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);

pub struct Peers
{
    // peer_map: HashMap<String, Ipv4Addr>
}

impl Peers
{
    pub async fn send_msg(&self, dest: Ipv4Addr, msg: String) -> Result<(), Box<dyn Error>>
    {
        let stream = TcpStream::connect((dest, PORT)).await?;
        let mut framed_conn = Framed::new(stream, LinesCodec::new());

        framed_conn.send(msg).await?;
        Ok(())
    }
    
    pub async fn listen_for_msg(&self) -> Result<(), Box<dyn Error>>
    {
        let addr = Ipv4Addr::new(0, 0, 0, 0);
        println!("this should be visible");
        let stream = TcpListener::bind((addr, PORT)).await?;
        
        loop {
            let (socket, addr) = stream.accept().await?;
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
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    let args: Vec<String> = std::env::args().collect();
    let mut peer_arg_iter =  args.iter();
    let _ =  peer_arg_iter.next().unwrap();
    let peer_arg =  peer_arg_iter.next().unwrap();
    
    let peers = Peers {
        // peer_map: HashMap::new(),
    };
    
    match peer_arg.as_str() {
        "PEER1" => {
            println!("PEER1");
            peers.send_msg(PEER2, String::from("hallo PEER2")).await?;
        },
        "PEER2" => {
            println!("PEER2");
            peers.listen_for_msg().await?;
        },
        _ => {}
    }
    Ok(())
}
