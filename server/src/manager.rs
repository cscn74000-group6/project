use crate::state_machine::{State, StateMachine};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio::time::{Duration, timeout};
use utils::packet::{FlagState, Packet, PacketHeader, deserialize_packet, serialize_packet};
use utils::vector::Vector3;

/// Type to asynchronously store/share the coordinates of active plane coordinates.
type Coordinates = Arc<Mutex<HashMap<u8, Vec<Vector3>>>>;

#[derive(Debug)]
pub struct Manager {
    coordinates: Coordinates,
    state_machine: StateMachine,
}

impl Manager {
    /// Create a new Manager.
    pub fn new() -> Manager {
        Manager {
            coordinates: Arc::new(Mutex::new(HashMap::new())),
            state_machine: StateMachine::new(),
        }
    }

    /// Main logic loop of the manager class
    pub async fn run(self) -> Result<(), std::io::Error> {
        // Listen into port 8001 on localhost
        let listener = TcpListener::bind("127.0.0.1:8001").await?;
        let (col_sender, _) = broadcast::channel::<(u8, f32)>(100);
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
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                Self::process_data(&coord_clone, &col_sender_clone).await;
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
        mut col_receiver: broadcast::Receiver<(u8, f32)>,
        exit_sender: mpsc::Sender<u8>,
        warn_sender: broadcast::Sender<u8>,
        mut warn_receiver: broadcast::Receiver<u8>,
    ) {
        let plane_id = match deserialize_packet(&mut stream).await {
            Ok(p) => {
                tracing::info!("Deserialized packet: {p}");
                p.header.plane_id
            }
            Err(e) => {
                tracing::error!("Error deserializing packet: {e}");
                return;
            }
        };

        loop {
            // Read packet from stream.
            let pkt = match timeout(Duration::from_secs(5), deserialize_packet(&mut stream)).await {
                Ok(Ok(p)) => {
                    tracing::info!("Deserialized packet: {p}");
                    p
                }
                Ok(Err(e)) => {
                    tracing::error!("Error deserializing packet: {e}");
                    return;
                }
                Err(_) => {
                    tracing::error!("Timed out waiting for packet");
                    if warn_sender.send(plane_id).is_err() {
                        tracing::error!("Error sending exit flag to manager...");
                    }

                    return;
                }
            };

            //packet handler
            match pkt.header.flag {
                FlagState::COORDINATE => {
                    tracing::info!("Packet is COORDINATE");
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
                FlagState::EXIT => {
                    //TODO: Handle massive load from client :weary:
                    tracing::info!("Packet is EXIT");
                    let mut file =
                        File::create(format!("plane_{}.txt", pkt.header.plane_id)).unwrap();

                    match file.write_all(&pkt.body) {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Failed to write final data to file... {}", e)
                        }
                    }

                    loop {
                        let pkt: Packet = match deserialize_packet(&mut stream).await {
                            Ok(p) => {
                                tracing::info!("Deserialized packet: {p}");
                                p
                            }
                            Err(e) => {
                                tracing::error!("Error deserializing exit packet: {e}");
                                return;
                            }
                        };

                        match file.write_all(&pkt.body) {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("Failed to write final data to file... {}", e)
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
                _ => {
                    tracing::error!(
                        "Something went terribly wrong, the server recieved a COLLISION packet..."
                    );
                }
            }

            // Check for collision warnings.
            // Send collision packet to affected clients.
            match col_receiver.recv().await {
                Ok(col_alert) => {
                    if col_alert.0 == pkt.header.plane_id {
                        let header = PacketHeader {
                            flag: FlagState::COLLISION,
                            plane_id: 0,
                            body_size: std::mem::size_of::<f32>() as u16,
                            seq_len: 0,
                        };
                        let new_altitude = Vector3::new(0.0, 0.0, col_alert.1);
                        let body = new_altitude.to_bytes();
                        let pkt = Packet { header, body };

                        if let Err(e) = serialize_packet(pkt, &mut stream) {
                            tracing::error!("Error sending packet: {e}");
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
                            seq_len: 0,
                        },
                        body: Vec::new(),
                    };

                    // Send WARNING packet.
                    if let Err(e) = serialize_packet(pkt, &mut stream) {
                        tracing::error!("Error sending packet: {e}");
                        return;
                    }
                }
                Ok(p) if p == plane_id => {
                    // Exit if this client timed out.
                    if exit_sender.send(pkt.header.plane_id).await.is_err() {
                        tracing::error!("Error sending exit flag to manager...");
                        return;
                    }
                }
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
    async fn process_data(
        coordinates: &Coordinates,
        col_sender: &Arc<Mutex<broadcast::Sender<(u8, f32)>>>,
    ) {
        // let sender = col_sender.lock().await;
        let data = coordinates.lock().await;
        if data.len() <= 0 {
            return;
        }

        let recent_coords: Vec<(Vector3, Vector3)> = data
            .values()
            .filter_map(|vec| {
                if vec.len() >= 2 {
                    let last_coord = vec[vec.len() - 2].clone();
                    let current_coord = vec.last().unwrap().clone();
                    Some((last_coord, current_coord))
                } else {
                    None
                }
            })
            .collect();

        // Check potential collisions with each plane
        for (i, plane_a) in recent_coords.iter().enumerate() {
            for (j, plane_b) in recent_coords.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let (prev_a, curr_a) = plane_a;
                let (prev_b, curr_b) = plane_b;

                let speed_a = Vector3::distance(*prev_a, *curr_a);
                let speed_b = Vector3::distance(*prev_b, *curr_b);

                let velocity_a = prev_a.displacement_vector(*curr_a, speed_a);
                let velocity_b = prev_b.displacement_vector(*curr_b, speed_b);

                // Send collision warnings if there will be a future collision.
                let max_cycles = 3;
                let tolerance = 2.0;
                if Vector3::will_intersect_in_n_cycles(
                    *curr_a, velocity_a, *curr_b, velocity_b, max_cycles, tolerance,
                ) {
                    let sender = col_sender.lock().await;
                    let plane_a_alert = (i as u8, 32000.0);
                    let plane_b_alert = (j as u8, 30000.0);
                    if sender.send(plane_a_alert).is_err() || sender.send(plane_b_alert).is_err() {
                        tracing::error!("Error sending collision alert to threads...");
                    }
                }
            }
        }
    }
}
