use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio::net::TcpStream;

use crate::server;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    /// Create a new client and connect to the server.
    pub async fn new() -> Result<Client> {
        match TcpStream::connect(server::ADDR).await {
            Ok(s) => Ok(Client { stream: s }),
            Err(e) => Err(e),
        }
    }

    /// Client's main loop.
    pub async fn run(&mut self) -> Result<()> {
        // Receive message from server
        let mut buf: Vec<u8> = vec![0; 1024];
        if let Err(e) = self.stream.read(&mut buf).await {
            eprintln!("{}", e);
            return Err(e);
        }

        println!("Received message: {}", String::from_utf8_lossy(&buf));

        // Send message to server
        let msg = "Hello, from the client!";
        if let Err(e) = self.stream.write_all(msg.as_bytes()).await {
            eprintln!("{}", e);
            return Err(e);
        }

        Ok(())
    }
}
