use crate::client_handler::ClientHandler;
use crate::coordinate::CoordinateData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task;
use utils::packet::{FlagState, PacketHeader, Packet, serialize_packet, deserialize_packet};
use utils::vector::Vector3;

/// Type to asynchronously store/share the coordinates of active plane coordinates.
type Coordinates = Arc<Mutex<HashMap<u8, Vec<Vector3>>>>;

#[derive(Debug)]
pub struct Manager {
    //pub clients: Vec<ClientHandler>,
    pub coordinates: Coordinates,
    pub col_warnings: HashMap<u8, u8>,
}

impl Manager {
    /// Create a new Manager.
    pub fn new() -> Manager {
        Manager {
            coordinates: Arc::new(Mutex::new(HashMap::new())),
            col_warnings: HashMap::new(),
        }
    }

    /// Main logic loop of the manager class
    pub async fn run(mut self) {
        let (_, warning_receiver) = watch::channel(self.col_warnings);
        // let (client_sender, client_receiver) = mpsc::channel::<ClientHandler>(100);

        // loop {
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

        //let listener_task = task::spawn(Self::accept_clients(
        //        listener,
        //        client_sender.clone(),
        //        warning_receiver.clone()));

        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                println!("Unable to accept client connection: {e}\nExiting now...");
                return;
            }
        };

        println!("Connected to client...");

        // Init client and client specific channels
        let (sender, receiver) = tokio::sync::mpsc::channel::<Vector3>(100);
        self.coord_data.push(CoordinateData::new(receiver));
        let mut client = ClientHandler::new(stream, warning_receiver.clone(), sender);

        task::spawn(async move {
            client.task().await;
        });

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
        //}
    }

    /// Receive and process packets from a client.
    pub async fn handle_client(
        mut stream: TcpStream,
        coordinates: Coordinates,
        mut warning_receiver: mpsc::Receiver<u8>,
        exit_sender: mpsc::Sender<bool>,
    ) {
        loop {
            // Check for collision warnings.
            if let Ok(w) = warning_receiver.try_recv() {
                let header = PacketHeader{
                    flag: FlagState::COLLISION,
                    plane_id: 0,
                    body_size: std::mem::size_of::<(u8, u8)>() as u16 
                };
                let body = vec![w];
                let pkt = Packet{
                    header,
                    body
                };

                if let Err(e) = serialize_packet(pkt, &mut stream) {
                    println!("Error sending packet: {e}");
                    return;
                }
            }

            // Read packet from stream.
            let pkt = match deserialize_packet(&mut stream).await {
                Ok(p) => p,
                Err(e) => {
                    println!("Error deserializing packet: {e}");
                    return;
                }
            };

            // Graceful exit.
            if pkt.header.flag == FlagState::EXIT {
                // Remove plane from active planes.
                {
                    let mut data = coordinates.lock().await;
                    if data.remove(&pkt.header.plane_id).is_none() {
                        println!("Unable to remove Plane #{} from active planes: entry not found",
                            pkt.header.plane_id);
                    }
                }

                // Send exit message to main thread.
                if exit_sender.send(true).await.is_err() {
                    println!("Error sending exit flag to manager...");
                }
            }

            // Read coordinates from packet body.
            let new_coord: Vector3 = match Vector3::from_bytes(pkt.body.as_slice()) {
                Some(c) => c,
                None => {
                    println!("Unable to create Vector3 from bytes...");
                    println!("Exiting task now...");
                    if exit_sender.send(true).await.is_err() {
                        println!("Error sending exit flag to manager...");
                    }
                    return;
                }
            };

            // Acquire lock, push new coordinate to shared HashMap.
            {
                let mut coord_data = coordinates.lock().await;
                coord_data.entry(pkt.header.plane_id)
                    .or_default()
                    .push(new_coord);
            }
        }
    }

    pub async fn accept_clients(
        listener: TcpListener,
        client_sender: mpsc::Sender<ClientHandler>,
        warning_receiver: watch::Receiver<HashMap<u8, u8>>,
    ) {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    println!("Connected to client...");

                    // Init client and client-specific channels
                    let (sender, _) = mpsc::channel::<Vector3>(100);
                    let client = ClientHandler::new(stream, warning_receiver.clone(), sender);

                    // Send client to the manager
                    if client_sender.send(client).await.is_err() {
                        println!("Manager dropped; shutting down listener...");
                        break;
                    }
                }
                Err(e) => {
                    println!("Unable to accept client connection: {e}");
                }
            }
        }
    }
}
