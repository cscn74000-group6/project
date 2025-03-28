use std::collections::HashMap;
use tokio::io::Result;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use utils::packet::deserialize_packet;
use utils::vector::Vector3;

#[derive(Debug)]
pub struct ClientHandler {
    pub plane_id: Option<u8>,
    pub stream: TcpStream,
    pub warning_receiver: Receiver<HashMap<u8, u8>>,
    pub coordinate_sender: Sender<Vector3>,
}

impl ClientHandler {
    /// Create a new ClientHandler.
    pub fn new(
        stream: TcpStream,
        warning_receiver: Receiver<HashMap<u8, u8>>,
        coordinate_sender: Sender<Vector3>,
    ) -> Self {
        Self {
            plane_id: None,
            stream,
            warning_receiver,
            coordinate_sender,
        }
    }

    /// The thread task for the ClientHandler. This is meant to be called by the spawn task function.
    pub async fn task(
        &mut self,
    ) -> Result<()> {
        let pkt = match deserialize_packet(&mut self.stream) {
            Ok(p) => p,
            Err(_) => {
                println!("Unable to deserialize_packet...");
                println!("Exiting task now...");
                return Ok(());
            }
        };

        let new_coord: Vector3 = match Vector3::from_bytes(pkt.body.as_slice()) {
            Some(c) => c,
            None => {
                println!("Unable to create Vector3 from bytes...");
                println!("Exiting task now...");
                return Ok(());
            }
        };

        if let Err(e) = self.coordinate_sender.send(new_coord).await {
            println!("Failed to send plane id: {e}");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }

        Ok(())
    }

    /// Spawn a thread that runs the ClientHandler task.
    pub async fn spawn_task(&mut self) {
        loop {
            match self.task().await {
                Ok(_) => println!("Task completed successfully"),
                // Check for warning
                // match self.warning_receiver.changed().await {
                //     Ok(_) => {
                //         // Process warning
                //     }
                //     Err(e) => {
                //         println!("Error: {e}");
                //     }
                // }
                Err(e) => println!("Error: {e}"),
            }
        }
    }
}
