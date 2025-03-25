use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;


#[tokio::main]
async fn main() {
    loop {
        match TcpStream::connect("127.0.0.1:8001").await {
            Ok(mut stream) => {
                println!("Successfully connected to server");
                match stream.write_u8(42).await {
                    Ok(_) => {
                        println!("Successfully sent data");
                    }
                    Err(e) => {
                        println!("Failed to write to server: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to server: {e}");
            }
        }
    }
}
