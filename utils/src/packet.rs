use tokio::net::TcpStream;
#[repr(u8)]
enum FlagState {
    WARNING = 0,
    COLLISION = 1,
    COORDINATE = 2,
    EXIT = 3,
}

impl FlagState {
    fn init(in_state: u8) -> FlagState {
        match in_state {
            0 => FlagState::WARNING,
            1 => FlagState::COLLISION,
            2 => FlagState::COORDINATE,
            3 => FlagState::EXIT,
            4_u8..=u8::MAX => {
                eprintln!("Invalid integer called for FlagState");
                return FlagState::WARNING;
            }
        }
    }
}

pub struct PacketHeader {
    flag: FlagState,
    plane_id: u8,
    body_size: u16,
}

impl PacketHeader {
    pub fn init() -> PacketHeader {
        return PacketHeader {
            flag: FlagState::COORDINATE,
            plane_id: 0,
            body_size: 0,
        };
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

// To use the vector return, just use &[variablename]
pub fn serialize_packet(pkt: Packet) -> Vec<u8> {
    let mut seralized_bytes: Vec<u8> = Vec::new();
    seralized_bytes.push(pkt.header.flag as u8);
    seralized_bytes.push(pkt.header.plane_id);
    seralized_bytes.push(pkt.header.body_size.try_into().unwrap());
    seralized_bytes.extend_from_slice(&pkt.body);
    return seralized_bytes;
}

pub fn deserialize_stream(stream: TcpStream) -> Packet {
    let mut rcv_buf_header: Vec<u8> = vec![0; std::mem::size_of::<PacketHeader>()];
    let mut pkt: Packet = Packet::init();
    let pkt_header = &mut pkt.header;

    // Read the data from the buffer
    if let Err(_e) = stream.try_read(&mut rcv_buf_header) {
        //eprintln!("{}", e);
        //return Err(e);
    }

    pkt_header.flag = FlagState::init(*rcv_buf_header.get(1).unwrap());
    pkt_header.plane_id = *rcv_buf_header.get(2).unwrap();
    pkt_header.body_size = u16::from_ne_bytes([
        *rcv_buf_header.get(3).unwrap(),
        *rcv_buf_header.get(4).unwrap(),
    ]);

    let mut rcv_buf: Vec<u8> = vec![0; pkt_header.body_size.into()];
    if let Err(_e) = stream.try_read(&mut rcv_buf) {
        //eprintln!("{}", e);
        //return Err(e);
    }
    pkt.body = rcv_buf;

    return pkt;
}
