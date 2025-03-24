use std::net::TcpStream;

enum FlagState {
    WARNING,
    COLLISION,
    COORDINATE,
    EXIT,
}

pub struct PacketHeader {
    flag: FlagState,
    plane_id: u8,
    body_size: u8,
}

impl PacketHeader {
    pub fn init() -> PacketHeader {
        return PacketHeader {
            flag: FlagState::COORDINATE,
            plane_id: 0,
            body_size: 0,
        };
    }
    // Hypothetical data stream, could just be a buffer read and passed to this
    fn read_packet_header(_stream: TcpStream) { // -> Self {
        //Self {
        //  self.flag = read(stream, 1)
        //  self.plane_id = read(stream, 1)
        //  self.body_size = read(stream, 1)
        //}
    }
}

pub struct Packet {
    header: PacketHeader,
    body: Vec<u8>,
}

impl Packet {
    pub fn init() -> Packet {
        return Packet {
            header: PacketHeader::init(),
            body: Vec::new(),
        };
    }
    fn read_body(_stream: TcpStream) {
        //self.body = read(stream, self.header.body_size)
    }
    pub fn get_pkt_type(&self) -> String {
        let ret: &str;
        match self.header.flag {
            FlagState::COORDINATE => ret = "COORDINATE",
            FlagState::EXIT => ret = "EXIT",
            FlagState::WARNING => ret = "WARNING",
            FlagState::COLLISION => ret = "COLLISION",
        }
        return String::from(ret);
    }
}
