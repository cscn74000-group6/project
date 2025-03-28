use crate::manager::Manager;
use tracing;
pub mod client_handler;
pub mod manager;

#[tokio::main]
async fn main() {
    let appender = tracing_appender::rolling::never("./log", "server.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_appender)
        .init();
    tracing::error!("Catch me!");
    let manager = Manager::new();
    manager.run().await;
}
