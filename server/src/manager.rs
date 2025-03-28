use crate::client_handler::ClientHandler;
use crate::coordinate::CoordinateData;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::watch;
use utils::vector::Vector3;

#[derive(Debug)]
pub struct Manager {
    pub clients: Vec<ClientHandler>,
    pub coord_data: Vec<CoordinateData>,
    pub col_warnings: HashMap<u8, u8>,
    // handles: Vec<dyn Future<Output=JoinHandle<_>>>,
}

impl Manager {
    /// Create a new Manager.
    pub fn new() -> Manager {
        Manager {
            clients: Vec::new(),
            col_warnings: HashMap::new(),
            coord_data: Vec::new(),
            // handles: Vec::new(),
        }
    }

    /// Main logic loop of the manager class
    pub async fn run(mut self) {
        let (_, warning_receiver) = watch::channel(self.col_warnings);

        loop {
            // Accept TCP connection, create client, asynchronously run client task
            let addr = "127.0.0.1:8001";
            let listener = match TcpListener::bind(addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    println!("Unable to bind to address: {e}\nExiting now...");
                    return;
                }
            };

            println!("Waiting for client connection on {addr}");
            let stream = match listener.accept().await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    println!("Unable to accept client connection: {e}\nExiting now...");
                    return;
                }
            };
            println!("Connected to client...");

            // Init client and client specific channels
            let (sender, receiver) = mpsc::channel::<Vector3>(100);
            self.coord_data.push(CoordinateData::new(receiver));
            let mut client = ClientHandler::new(
                stream,
                warning_receiver.clone(),
                sender,
            );

            let _ = client.spawn_task();
            self.clients.push(client);

            // Iterate over coordinate receivers, update values.
            for c in self.coord_data.iter_mut() {
                let v = match c.receiver.recv().await {
                    Some(v) => v,
                    None => {
                        println!("Error: unable to receive coordinate from client_handler...");
                        return;
                    }
                };

                println!("[COORD] {v}");
                c.coordinates.push(v);
            }

            //self.receivers.as_slice()
            //    .into_iter()
            //    .map(|r| {
            //        if 
            //    });
        }
    }

    //pub async fn 
}


