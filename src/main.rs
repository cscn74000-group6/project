pub mod client;
pub mod server;

use std::env;

use client::Client;
use server::Server;

use tokio::io::Result;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        print_usage()
    }

    match args[1].as_str() {
        "server" => {
            let _ = run_server().await;
        }
        "client" => {
            let _ = run_client().await;
        }
        _ => print_usage(),
    }
}

/// Print program usage.
fn print_usage() {
    println!("USAGE");
    println!("project.exe [server | client]")
}

/// Create and run a server.
async fn run_server() -> Result<()> {
    let listener = TcpListener::bind(server::ADDR).await?;
    let mut s = Server::new(listener);
    if let Ok(message) = s.run().await {
        println!("Received message: {}", message);
    }
    println!("Server completed task");
    Ok(())
}

/// Create and run a client.
async fn run_client() -> Result<()> {
    let mut client = match Client::new().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            panic!("client exploded")
        }
    };

    let _ = client.run().await?;
    println!("Client completed task");
    Ok(())
}
