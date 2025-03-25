use crate::manager::Manager;

pub mod client_handler;
pub mod manager;

#[tokio::main]
async fn main() {
    let manager = Manager::new();
    manager.run().await;
}
