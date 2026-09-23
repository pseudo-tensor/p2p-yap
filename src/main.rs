mod peers;

use std::{
    collections::HashMap,
    error::Error,
};
use crate::peers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>
{
    let mut peers = Peers {
        peer_map: HashMap::new(),
        channel: None,
    };
    
    // Listener only thread
    tokio::spawn(async move {
        let _ = listen().await;
    });

    let mut exit_flag = false;
    
    let mut buf = String::new();
    while !exit_flag {
        // PONDER: Possibility of injection here?
        let _ = std::io::stdin().read_line(&mut buf)?;
        exit_flag = process_command(&mut peers, &mut buf).await?;
    }

    Ok(())
}
