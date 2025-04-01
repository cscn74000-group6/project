use crate::manager::Manager;

pub mod manager;

#[tokio::main]
async fn main() {
    match Manager::new().run().await {
        Ok(_) => {
            println!("Manager exited gracefully...");
        }
        Err(e) => {
            eprintln!("Manager exitied with error: {e}");
        }
    }
}
