use crate::manager::Manager;

pub mod client_handler;
pub mod coordinate;
pub mod manager;

#[tokio::main]
async fn main() {
    Manager::new().run().await;
    // let addr = "127.0.0.1:8001";
    // let listener = match TcpListener::bind(addr).await {
    //     Ok(listener) => listener,
    //     Err(e) => {
    //         println!("Unable to bind to address: {e}\nExiting now...");
    //         return;
    //     }
    // };
    //
    // println!("Waiting for client connection on {addr}");
    // let stream = match listener.accept().await {
    //     Ok((stream, _)) => stream,
    //     Err(e) => {
    //         println!("Unable to accept client connection: {e}\nExiting now...");
    //         return;
    //     }
    // };
    // println!("Connected to client...");
    //
    // // Init client and client specific channels
    // let (_, warning_receiver) = watch::channel(m.col_warnings);
    // let (sender, receiver) = mpsc::channel::<Vector3>(100);
    // m.coord_data.push(CoordinateData::new(receiver));
    // let mut client = ClientHandler::new(stream, warning_receiver.clone(), sender);
    //
    // // m.clients.push(client);
    // task::spawn(async move {
    //     client.task().await;
    // });
    //
    // loop {
    //     // Iterate over coordinate receivers, update values.
    //     for c in m.coord_data.iter_mut() {
    //         let v = match c.receiver.recv().await {
    //             Some(v) => v,
    //             None => {
    //                 println!("Error: unable to receive coordinate from client_handler...");
    //                 return;
    //             }
    //         };
    //
    //         println!("[COORD] {v}");
    //         c.coordinates.push(v);
    //     }
    // }
}
