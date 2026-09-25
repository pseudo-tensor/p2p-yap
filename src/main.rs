mod peers;

use std::{
    collections::HashMap,
    error::Error,
};
use tokio::sync::{mpsc, oneshot};
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
    
    let (tx, mut rx) = mpsc::channel::<Command>(32);
    
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
        
        /*
         * Listen thread took longer to receive the message than this block finished 
         * and process_command started execution;
         * 
         * Now it is blocked until the confirmation arrives
         */
        if tx_flag
        {
            let (ack_tx, ack_rx) = oneshot::channel();
            let cmd = Command {
                show: true,
                respond_to: ack_tx,
            };
            let res = tx.send(cmd).await;
            // should be a shorthand for this
            match res {
                Ok(()) => {},
                Err(_) => {},
            }
            ack_rx.await?;
        }
        
        (exit_flag, tx_flag) = process_command(&mut peers, &user_nick).await?;
    }

    Ok(())
}
