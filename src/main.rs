mod peers;

use std::{
    collections::HashMap, error::Error, io::Write,
};
use tokio::sync::mpsc;

use crate::peers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    let mut peers = Peers {
        peer_map: HashMap::new(),
        channel: None,
    };

    let mut inbox = Inbox {
        inbox: vec![],
    };
    
    let (tx, mut rx) = mpsc::channel(32);
    
    // Listener only thread
    tokio::spawn(async move {
        let _ = listen(&mut rx, &mut inbox).await;
    });

    let mut exit_flag = false;
    let mut tx_flag = false;
    
    while !exit_flag {
        let user_nick: String;
        
        match &peers.channel
        {
            Some((_, nick)) => user_nick = String::from(nick),
            None => user_nick = String::from(""),
        }
        
        if tx_flag
        {
            let res = tx.send("show").await;
            match res {
                Ok(()) => {},
                Err(_) => println!("Internal Error, pls try again"),
            }
        }
        
        print!("[{user_nick}]> ");
        let _ = std::io::stdout().flush();
        let mut buf = String::new();
        
        // PONDER: Possibility of injection here?
        let _ = std::io::stdin().read_line(&mut buf)?;
        (exit_flag, tx_flag) = process_command(&mut peers, &mut buf).await?;
    }

    Ok(())
}
