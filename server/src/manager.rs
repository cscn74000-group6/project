use crate::client_handler::ClientHandler;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::watch;

#[derive(Debug)]
pub struct Manager {
    clients: Vec<ClientHandler>,
    receivers: Vec<mpsc::Receiver<u8>>,
    warnings: HashMap<u8, u8>,
    // plane_coordinates: Vec<Vec<Vector3>>,
    // handles: Vec<dyn Future<Output=JoinHandle<_>>>,
}

impl Manager {
    /// Create a new Manager.
    pub fn new() -> Manager {
        Manager {
            clients: Vec::new(),
            receivers: Vec::new(),
            warnings: HashMap::new(),
            // handles: Vec::new(),
        }
    }

    /// Main logic loop of the manager class
    pub async fn run(mut self) {
        let (_, warning_receiver) = watch::channel(self.warnings);

        loop {
            // Accept TCP connection, create client, asynchronously run client task
            match TcpListener::bind("127.0.0.1:8001").await {
                Ok(listener) => {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            // Init client and client specific channels
                            let (sender, receiver) = mpsc::channel::<u8>(100);
                            self.receivers.push(receiver);
                            let mut client = ClientHandler::new(
                                stream,
                                warning_receiver.clone(),
                                sender,
                            );

                            let _ = client.spawn_task();
                            self.clients.push(client);
                        }
                        Err(e) => {
                            println!("Error: {e}");
                            return;
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {e}");
                    return;
                }
            }
        }
    }
}


