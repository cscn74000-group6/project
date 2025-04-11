use crate::manager::Manager;
use tracing;
pub mod manager;
pub mod state_machine;

#[tokio::main]
async fn main() {
    let appender = tracing_appender::rolling::never("./server/log", "server.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .init();

    // Initialize and run server manager.
    match Manager::new().run().await {
        Ok(_) => {
            tracing::info!("Manager exited gracefully...");
        }
        Err(e) => {
            tracing::error!("Manager exitied with error: {e}");
        }
    }
}
