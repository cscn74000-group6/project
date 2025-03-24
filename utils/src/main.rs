use packet::Packet;

pub mod packet;

fn main() {
    let pkt: Packet = Packet::init();
    println!("{}", pkt.get_pkt_type());
}
