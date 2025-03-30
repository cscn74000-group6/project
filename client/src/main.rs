use std::env;
use std::{thread, time};
use tokio::net::TcpStream;
use utils::packet::{FlagState, Packet, PacketHeader, serialize_packet};
use utils::vector::Vector3;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id: String = args[1].clone();
    let start_pos: Vector3 = Vector3::new(
        args[2].clone().parse::<f32>().unwrap(),
        args[3].clone().parse::<f32>().unwrap(),
        args[4].clone().parse::<f32>().unwrap(),
    );
    let end_pos = Vector3::new(
        args[5].clone().parse::<f32>().unwrap(),
        args[6].clone().parse::<f32>().unwrap(),
        args[7].clone().parse::<f32>().unwrap(),
    );
    let plane_speed = args[8].clone().parse::<f32>().unwrap();

    let mut plane_pos = start_pos;

    //enter flight loop
    //loop {
    println!("Connecting to server...");
    let mut stream = match TcpStream::connect("127.0.0.1:8001").await {
        Ok(listener) => listener,
        Err(_) => {
            println!("Unable to connect to server...\nExiting now...");
            return;
        }
    };
    println!("Connected to server!");

    loop {
        //move aircraft
        plane_pos = plane_pos.add(plane_pos.displacement_vector(end_pos, plane_speed));
        println!("{client_id} moved to {plane_pos}");

        //if distance to destination is less than A VALUE (idk what)
        // if Vector3::distance(plane_pos, end_pos) < 10.0 {
        //     return;
        // }

        // Initialize packet
        let body = plane_pos.to_bytes();
        let header = PacketHeader {
            flag: FlagState::COORDINATE,
            plane_id: 1,
            body_size: body.len() as u16,
        };
        let pkt = Packet { header, body };

        // Serialize and send packet
        if let Err(e) = serialize_packet(pkt, &mut stream) {
            println!("Error sending packet: {e}");
            return;
        }

        //send data
        println!("Packet sent...");

        //wait for 5 seconds
        let ten_millis = time::Duration::from_secs(1);
        thread::sleep(ten_millis);
    }

    //send big data

    // TCP DEMO CODE
    // use tokio::io::AsyncWriteExt;
    // use tokio::net::TcpStream;

    // #[tokio::main]
    // async fn main() {
    //     loop {
    //         match TcpStream::connect("127.0.0.1:8001").await {
    //             Ok(mut stream) => {
    //                 println!("Successfully connected to server");
    //                 match stream.write_u8(42).await {
    //                     Ok(_) => {
    //                         println!("Successfully sent data");
    //                     }
    //                     Err(e) => {
    //                         println!("Failed to write to server: {e}");
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 println!("Failed to connect to server: {e}");
    //             }
    //         }
    //     }
}
