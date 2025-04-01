use std::env;
use std::fs::File;
use std::io::Read;
use std::{thread, time};
use tokio::net::TcpStream;
use utils::packet::{FlagState, Packet, PacketHeader, serialize_packet};
use utils::vector::Vector3;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let client_id: u8 = args[1].clone().parse::<u8>().unwrap();
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

    //Initalize the filename of the client here for logging
    let log_filename = format!("client-{}.log", client_id);
    let appender = tracing_appender::rolling::never("./client/log", log_filename);
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_appender)
        .init();

    //enter flight loop
    //loop {
    tracing::info!("Connecting to server...");
    let mut stream = match TcpStream::connect("127.0.0.1:8001").await {
        Ok(listener) => listener,
        Err(_) => {
            tracing::error!("Unable to connect to server...\nExiting now...");
            return;
        }
    };
    tracing::info!("Connected to server!");

        loop {
            //move aircraft
            plane_pos = plane_pos.add(plane_pos.displacement_vector(end_pos, plane_speed));
            tracing::info!("{client_id} moved to {plane_pos}");

            // if distance to destination is less than A VALUE (idk what) (probably unhardcode this)
            if Vector3::distance(plane_pos, end_pos) < 10.0 {
                break;
            }

            // Initialize packet
            let body = plane_pos.to_bytes();
            let header = PacketHeader {
                flag: FlagState::COORDINATE,
                plane_id: client_id,
                body_size: body.len() as u16,
                seq_len: 0,
            };
            let pkt = Packet { header, body };

        // Serialize and send packet
        if let Err(e) = serialize_packet(pkt, &mut stream) {
            tracing::error!("Error sending packet: {e}");
            return;
        }

        //send data
        tracing::info!("Packet sent...");

            //wait for 5 seconds
            let ten_millis = time::Duration::from_secs(5);
            thread::sleep(ten_millis);
        }

        //send big data
        tracing::info!("Flight Done, sending big packet...");

        let mut file = File::open("test.txt").unwrap();
        let mut buf = Vec::new();

        match file.read_to_end(&mut buf) {
            Ok(s) => {
                println!("Read {s} bytes");
            }
            Err(e) => {
                panic!("File exploded \n{e}")
            }
        }

        let str_buf = String::from_utf8(buf).unwrap();
        let chunk_size = 65500;
        let mut count = 0;
        let segments = str_buf.len() / chunk_size;
        // let test = segments-count;
        //count = number of segments sent SO FAR
        //segments = number of segments to send TOTAL

        //while the number of sent segments is less than or equal to the number to send
        while count <= segments {
            //chose end point
            let end = std::cmp::min(count * chunk_size + chunk_size, str_buf.len());
            //chose start point
            let start = count * chunk_size;
            dbg!(start, end);
            let chunk = &str_buf[start..end];

            // Initialize packet
            let final_body = chunk.as_bytes().to_vec();
            let final_header = PacketHeader {
                flag: FlagState::EXIT,
                plane_id: client_id,
                body_size: final_body.len() as u16,
                seq_len: (segments - count) as u8,
            };
            let final_pkt = Packet {
                header: final_header,
                body: final_body,
            };

            // Serialize and send packet
            if let Err(e) = serialize_packet(final_pkt, &mut stream) {
                println!("Error sending packet: {e}");
                return;
            }

            //increment the counter
            count += 1;
        }
    }
    println!("Done, exiting...");

    //wait for 5 seconds
    let ten_millis = time::Duration::from_secs(5);
    thread::sleep(ten_millis);
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
