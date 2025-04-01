use crate::state_machine::{State, StateMachine};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc, watch};
use utils::packet::{FlagState, Packet, PacketHeader, deserialize_packet, serialize_packet};
use utils::vector::Vector3;

/// Type to asynchronously store/share the coordinates of active plane coordinates.
type Coordinates = Arc<Mutex<HashMap<u8, Vec<Vector3>>>>;

#[derive(Debug)]
pub struct Manager {
     coordinates: Coordinates,
     col_warnings: HashMap<u8, u8>,
     state_machine: StateMachine
}

impl Manager {
    /// Create a new Manager.
    pub fn new() -> Manager {
        Manager {
            coordinates: Arc::new(Mutex::new(HashMap::new())),
            col_warnings: HashMap::new(),
            state_machine: StateMachine::new(),
        }
    }

    /// Main logic loop of the manager class
    pub async fn run(self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind("127.0.0.1:8001").await?;
        let (warning_sender, _) = watch::channel((0, 0));
        let (exit_sender, mut exit_receiver) = mpsc::channel::<u8>(100);

        // Spawn task to handle client exits.
        let coord_clone = self.coordinates.clone();
        tokio::spawn(async move {
            while let Some(plane_id) = exit_receiver.recv().await {
                println!("Client {} disconnected", plane_id);
                let mut state = coord_clone.lock().await;
                state.remove(&plane_id);
            }
        });

        // Spawn task to process new data.
        let coord_clone = self.coordinates.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                Self::process_all_data(&coord_clone).await;
            }
        });

        // Listen for new client connections.
        // Spawn task to handle client.
        loop {
            match self.state_machine.get_state() {
                State::OPEN => {
                    let (stream, addr) = listener.accept().await?;
                    println!("New client connected: {}", addr);

                    let coord_clone = self.coordinates.clone();
                    let exit_sender = exit_sender.clone();
                    tokio::spawn(Self::handle_client(
                        stream,
                        coord_clone,
                        warning_sender.subscribe(),
                        exit_sender,
                    ));
                },
                State::CLOSED => {}
            }
        }
    }

    /// Receive and process packets from a client.
    pub async fn handle_client(
        mut stream: TcpStream,
        coordinates: Coordinates,
        mut warning_receiver: watch::Receiver<(u8, u8)>,
        exit_sender: mpsc::Sender<u8>,
    ) {
        loop {
            // Read packet from stream.
            let pkt = match deserialize_packet(&mut stream).await {
                Ok(p) => p,
                Err(e) => {
                    println!("Error deserializing packet: {e}");
                    return;
                }
            };

            // Check for collision warnings. Send collision packet to client if client for this
            // plane is created.
            match warning_receiver.has_changed() {
                Ok(true) => {
                    let warning = &warning_receiver.borrow_and_update();
                    if warning.0 == pkt.header.plane_id {
                        let header = PacketHeader {
                            flag: FlagState::COLLISION,
                            plane_id: 0,
                            body_size: std::mem::size_of::<(u8, u8)>() as u16,
                        };
                        let body = vec![];
                        let pkt = Packet { header, body };

                        if let Err(e) = serialize_packet(pkt, &mut stream) {
                            println!("Error sending packet: {e}");
                            return;
                        }
                    }
                },
                Err(_) => {
                    println!("Error reading warning from mananger...");
                    return;
                },
                _ => {}
            };

            // Handle EXIT flag.
            if pkt.header.flag == FlagState::EXIT {
                // Remove plane from active planes.
                {
                    let mut data = coordinates.lock().await;
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
    }

    /// Process data.
    async fn process_all_data(coordinates: &Coordinates) {
        let data = coordinates.lock().await;
        println!("--- Processing all client data ---");
        for (client, messages) in data.iter() {
            println!("Client [{}]: {:?}", client, messages);
        }
    }
}
