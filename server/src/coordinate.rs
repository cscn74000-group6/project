use tokio::sync::mpsc;
use utils::vector::Vector3;

#[derive(Debug)]
pub struct CoordinateData {
    pub receiver: mpsc::Receiver<Vector3>, 
    pub coordinates: Vec<Vector3>, 
}

impl CoordinateData {
    pub fn new(receiver: mpsc::Receiver<Vector3>) -> CoordinateData {
        CoordinateData {
            receiver,
            coordinates: Vec::new(),
        }
    }
}
