use std::io::{Error, ErrorKind};

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::Result;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

pub const ADDR: &str = "127.0.0.1:8001";

pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Create a new server from a TCP listener.
    pub fn new(listener: TcpListener) -> Server {
        Server { listener }
    }

    /// Server's main loop.
    pub async fn run(&mut self) -> Result<String> {

        let stream = self.accept_connection().await?;
        let join_handle = tokio::spawn(async move {
            Self::communicate(stream).await.unwrap_or_else(|e| e.to_string())
        });

        let result = join_handle.await?;
        Ok(result)
    }

    /// Accept connection over listener. Expects listener to be initialized.
    pub async fn accept_connection(&mut self) -> Result<TcpStream> {
        let (sock, _) = self.listener.accept().await?;
        Ok(sock)
    }

    /// Send and receive a message with the client.
    pub async fn communicate(mut stream: TcpStream) -> Result<String> {
        let msg = "Hello, from the server!";
        if let Err(e) = stream.write(msg.as_bytes()).await {
            eprintln!("{}", e);
            return Err(e);
        }

        let mut rcv_buf: Vec<u8> = vec![0; 1024];
        if let Err(e) = stream.read(&mut rcv_buf).await {
            eprintln!("{}", e);
            return Err(e);
        }

        match String::from_utf8(rcv_buf) {
            Ok(msg) => Ok(msg),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }
}
