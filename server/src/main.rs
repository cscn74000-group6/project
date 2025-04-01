use crate::manager::Manager;

pub mod manager;
pub mod state_machine;

#[tokio::main]
async fn main() {
    // Initialize and run server manager.
    match Manager::new().run().await {
        Ok(_) => {
            println!("Manager exited gracefully...");
        }
        Err(e) => {
            eprintln!("Manager exitied with error: {e}");
        }
    }
}
