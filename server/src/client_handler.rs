use std::collections::HashMap;
use tokio::io::{AsyncReadExt, Result};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;

#[derive(Debug)]
pub struct ClientHandler {
    pub plane_id: Option<u8>,
    pub stream: TcpStream,
    pub warning_receiver: Receiver<HashMap<u8, u8>>,
    pub coordinate_sender: Sender<u8>,
}

impl ClientHandler {
    /// Create a new ClientHandler.
    pub fn new(
        stream: TcpStream,
        warning_receiver: Receiver<HashMap<u8, u8>>,
        coordinate_sender: Sender<u8>,
    ) -> Self {
        Self {
            plane_id: None,
            stream,
            warning_receiver,
            coordinate_sender,
        }
    }

    /// !STILL IN PROGRESS!
    ///
    /// The thread task for the ClientHandler. This is meant to be called by the spawn task function.
    pub async fn task(
        &mut self,
    ) -> Result<()> {
        let id = self.stream.read_u8().await?;
        self.plane_id = Some(id);

        if let Err(e) = self.coordinate_sender.send(id).await {
            println!("Failed to send plane id: {e}");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }

        Ok(())
    }

    /// !STILL IN PROGRESS!
    ///
    /// Spawn a thread that runs the ClientHandler task.
    pub async fn spawn_task(&mut self) {
        loop {
            match self.task().await {
                Ok(_) => {
                    // Check for warning
                    match self.warning_receiver.changed().await {
                        Ok(_) => {
                            // Process warning
                        }
                        Err(e) => {
                            println!("Error: {e}");
                        }
                    }
                }
                Err(e) => println!("Error: {e}"),
            }
        }
    }
}
