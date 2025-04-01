use crate::state_machine::{State, StateMachine};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc, watch, broadcast};
use tokio::time::{timeout, Duration};
use utils::packet::{FlagState, Packet, PacketHeader, deserialize_packet, serialize_packet};
use utils::vector::Vector3;

/// Type to asynchronously store/share the coordinates of active plane coordinates.
type Coordinates = Arc<Mutex<HashMap<u8, Vec<Vector3>>>>;

#[derive(Debug)]
pub struct Manager {
    coordinates: Coordinates,
    //col_warnings: HashMap<u8, u8>,
    state_machine: StateMachine,
}

impl Manager {
    /// Create a new Manager.
    pub fn new() -> Manager {
        Manager {
            coordinates: Arc::new(Mutex::new(HashMap::new())),
            //col_warnings: HashMap::new(),
            state_machine: StateMachine::new(),
        }
    }

    /// Main logic loop of the manager class
    pub async fn run(self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind("127.0.0.1:8001").await?;
        let (col_sender, _) = broadcast::channel::<u8>(100);
        let (warn_sender, _) = broadcast::channel::<u8>(100);
        let (exit_sender, mut exit_receiver) = mpsc::channel::<u8>(100);
        let col_sender = Arc::new(Mutex::new(col_sender));

        // Spawn task to handle client exits.
        let coord_clone = self.coordinates.clone();
        tokio::spawn(async move {
            while let Some(plane_id) = exit_receiver.recv().await {
                tracing::info!("Client {} disconnected", plane_id);
                let mut data = coord_clone.lock().await;
                data.remove(&plane_id);
            }
        });


        // Spawn task to process new data.
        let coord_clone = self.coordinates.clone();
        let col_sender_clone = Arc::clone(&col_sender);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                Self::process_all_data(&coord_clone, &col_sender_clone).await;
            }
        });

        // Listen for new client connections.
        // Spawn task to handle client.
        loop {
            match self.state_machine.get_state() {
                State::OPEN => {
                    let (stream, addr) = listener.accept().await?;
                    tracing::info!("New client connected: {}", addr);
                    let coord_clone = self.coordinates.clone();
                    let col_sender_clone = Arc::clone(&col_sender);
                    let col_receiver = {
                        let sender = col_sender.lock().await;
                        sender.subscribe()
                    };
                    let exit_sender = exit_sender.clone();
                    let warn_sender = warn_sender.clone();
                    let warn_receiver = warn_sender.subscribe(); 
                    tokio::spawn(Self::handle_client(
                        stream,
                        coord_clone,
                        col_receiver,
                        exit_sender,
                        warn_sender,
                        warn_receiver,
                    ));
                }
                State::CLOSED => {}
            }
        }
    }

    /// Receive and process packets from a client.
    pub async fn handle_client(
        mut stream: TcpStream,
        coordinates: Coordinates,
        mut col_receiver: broadcast::Receiver<u8>,
        exit_sender: mpsc::Sender<u8>,
        warn_sender: broadcast::Sender<u8>,
        mut warn_receiver: broadcast::Receiver<u8>
    ) {
        let plane_id = match deserialize_packet(&mut stream).await {
            Ok(p) => p.header.plane_id,
            Err(e) => {
                println!("Error deserializing packet: {e}");
                return;
            }
        }; 
        loop {
            // Read packet from stream.
            let pkt = match timeout(Duration::from_secs(5), deserialize_packet(&mut stream)).await {
                Ok(Ok(p)) => p,
                Ok(Err(e)) => {
                    tracing::error!("Error deserializing packet: {e}");
                    return;
                },
                Err(_) => {
                    tracing::error!("Timed out waitng for packet");
                    if warn_sender.send(plane_id).is_err() {
                        tracing::error!("Error sending exit flag to manager...");
                    }

                    return;
                }
            };

            //packet handler
            match pkt.header.flag {
                FlagState::COORDINATE => {
                    // Read coordinates from packet body.
                    let new_coord: Vector3 = match Vector3::from_bytes(pkt.body.as_slice()) {
                        Some(c) => c,
                        None => {
                            println!("Unable to create Vector3 from bytes...");
                            println!("Exiting task now...");
                            if exit_sender.send(pkt.header.plane_id).await.is_err() {
                                println!("Error sending exit flag to manager...");
                            }
                            return;
                        }
                    };

                    // Acquire lock, push new coordinate to shared HashMap.
                    {
                        let mut coord_data = coordinates.lock().await;
                        coord_data
                            .entry(pkt.header.plane_id)
                            .or_default()
                            .push(new_coord);
                    }
                }
                FlagState::EXIT => {
                    //TODO: Handle massive load from client :weary:
                    let mut file =
                        File::create(format!("plane_{}.txt", pkt.header.plane_id)).unwrap();

                    match file.write_all(&pkt.body) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Failed to write final data to file... {}", e)
                        }
                    }

                    loop {
                        let pkt: Packet = match deserialize_packet(&mut stream).await {
                            Ok(p) => p,
                            Err(e) => {
                                println!("Error deserializing exit packet: {e}");
                                return;
                            }
                        };

                        match file.write_all(&pkt.body) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Failed to write final data to file... {}", e)
                            }
                        }

                        if pkt.header.seq_len == 0 {
                            break;
                        }
                    }

                    // Remove plane from active planes.
                    {
                        let mut data: tokio::sync::MutexGuard<'_, HashMap<u8, Vec<Vector3>>> =
                            coordinates.lock().await;
                        if data.remove(&pkt.header.plane_id).is_none() {
                            println!(
                                "Unable to remove Plane #{} from active planes: entry not found",
                                pkt.header.plane_id
                            );
                        }
                    }

                    // Send exit message to main thread.
                    if exit_sender.send(pkt.header.plane_id).await.is_err() {
                        println!("Error sending exit flag to manager...");
                    }
                }
                FlagState::COLLISION => {
                    println!(
                        "Something went terribly wrong, the server recieved a COLLISION packet..."
                    );
                }
                FlagState::WARNING => {
                    println!(
                        "Something went terribly wrong, the server recieved a WARNING packet..."
                    );
                }
            }

            // Check for collision warnings. Send collision packet to client if client for this
            // plane is created.
            match col_receiver.recv().await {
                Ok(warning) => {
                    if warning == pkt.header.plane_id {
                        let header = PacketHeader {
                            flag: FlagState::COLLISION,
                            plane_id: 0,
                            body_size: std::mem::size_of::<u8>() as u16,
                            seq_len: 0
                        };
                        let body = vec![];
                        let pkt = Packet { header, body };

                        if let Err(e) = serialize_packet(pkt, &mut stream) {
                            println!("Error sending packet: {e}");
                            return;
                        }
                    }
                }
                Err(_) => {
                    tracing::error!("Error reading warning from mananger...");
                    return;
                }
            };

            // Check for timeout warnings.
            match warn_receiver.recv().await {
                Ok(p) if p != plane_id => {
                    // Create WARNING packet.
                    let pkt = Packet {
                            header: PacketHeader {
                            flag: FlagState::WARNING,
                            plane_id: p,
                            body_size: 0 as u16,
                            seq_len: 0
                        },
                        body: Vec::new(),
                    };  

                    // Send WARNING packet.
                    if let Err(e) = serialize_packet(pkt, &mut stream) {
                        tracing::error!("Error sending packet: {e}");
                        return;
                    }
                },
                Ok(p) if p == plane_id => {
                    // Exit if this client timed out.
                    if exit_sender.send(pkt.header.plane_id).await.is_err() {
                        tracing::error!("Error sending exit flag to manager...");
                        return;
                    }
                },
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Unable to broadcast timeout warning {}", e);
                }
            }

            // Handle EXIT flag.
            if pkt.header.flag == FlagState::EXIT {
                // Remove plane from active planes.
                {
                    let mut data = coordinates.lock().await;
                    if data.remove(&pkt.header.plane_id).is_none() {
                        tracing::error!(
                            "Unable to remove Plane #{} from active planes: entry not found",
                            pkt.header.plane_id
                        );
                    }
                }

                // Send exit message to main thread.
                if exit_sender.send(pkt.header.plane_id).await.is_err() {
                    tracing::error!("Error sending exit flag to manager...");
                }
            }

            // Read coordinates from packet body.
            let new_coord: Vector3 = match Vector3::from_bytes(pkt.body.as_slice()) {
                Some(c) => c,
                None => {
                    tracing::error!("Unable to create Vector3 from bytes...");
                    tracing::error!("Exiting task now...");
                    if exit_sender.send(pkt.header.plane_id).await.is_err() {
                        tracing::error!("Error sending exit flag to manager...");
                    }
                    return;
                }
            };

            // Acquire lock, push new coordinate to shared HashMap.
            {
                let mut coord_data = coordinates.lock().await;
                coord_data
                    .entry(pkt.header.plane_id)
                    .or_default()
                    .push(new_coord);
            }
        }
    }

    /// Process data.
    async fn process_all_data(
        coordinates: &Coordinates,
        col_sender: &Arc<Mutex<broadcast::Sender<u8>>>,
        ) {
        let mut sender = col_sender.lock().await;
        let data = coordinates.lock().await;
        tracing::info!("--- Processing all client data ---");
        let latest_coords: Vec<(Vector3, Vector3)> = data.values()
            .filter_map(|vec| {
                if vec.len() >= 2 {
                    Some((vec[vec.len() - 2].clone(), vec.last().unwrap().clone()))
                } else {
                    None
                }
            })
            .collect();

        let danger_distance = 10.0;
        for (i, &plane_a) in latest_coords.iter().enumerate() {
            for &plane_b in &latest_coords[i + 1..] {
                let vec_a = plane_a.1.displacement_vector(plane_a.0, 1.0);
                let vec_b = plane_b.1.displacement_vector(plane_b.0, 1.0);
                if let Some(col_point) = Vector3::intersection(plane_a.1, vec_a, plane_b.1, vec_b) {
                    if Vector3::distance(vec_a, col_point) <= danger_distance
                        || Vector3::distance(vec_b, col_point) <= danger_distance {
                        if sender.send(i as u8).is_err() {
                            tracing::error!("Error sending collision flag to manager...");
                            return;
                        }
                    }
                };  
            }
        }
    }
}
