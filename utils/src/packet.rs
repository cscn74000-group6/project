use std::fmt;
use tokio::net::TcpStream;
#[repr(u8)]
pub enum FlagState {
    WARNING = 0,
    COLLISION = 1,
    COORDINATE = 2,
    EXIT = 3,
}

impl FlagState {
    pub fn init(in_state: u8) -> FlagState {
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
    pub flag: FlagState,
    pub plane_id: u8,
    pub body_size: u16,
}

impl PacketHeader {
    pub fn init() -> PacketHeader {
        return PacketHeader {
            flag: FlagState::COORDINATE,
            plane_id: 0,
            body_size: 0,
        };
    }
    fn deseralize_packet_header(stream: Vec<u8>) -> PacketHeader {
        return PacketHeader {
            flag: FlagState::init(*stream.get(1).unwrap()),
            plane_id: *stream.get(2).unwrap(),
            body_size: u16::from_ne_bytes([*stream.get(3).unwrap(), *stream.get(4).unwrap()]),
        };
    }
}

pub struct Packet {
    pub header: PacketHeader,
    pub body: Vec<u8>,
}

impl Packet {
    pub fn init() -> Packet {
        return Packet {
            header: PacketHeader::init(),
            body: Vec::new(),
        };
    }
}

impl fmt::Display for FlagState {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ret: &str;
        match self {
            FlagState::COORDINATE => ret = "COORDINATE",
            FlagState::EXIT => ret = "EXIT",
            FlagState::WARNING => ret = "WARNING",
            FlagState::COLLISION => ret = "COLLISION",
        }
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", String::from(ret))
    }
}

impl fmt::Display for PacketHeader {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{0}, {1}, {2}]",
            self.flag, self.plane_id, self.body_size
        )
    }
}

impl fmt::Display for Packet {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{0}\n{1}",
            self.header,
            String::from_utf8(self.body.clone()).unwrap()
        )
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

    // Read the data from the buffer
    if let Err(_e) = stream.try_read(&mut rcv_buf_header) {
        //eprintln!("{}", e);
        //return Err(e);
    }

    pkt.header = PacketHeader::deseralize_packet_header(rcv_buf_header);

    let mut rcv_buf: Vec<u8> = vec![0; pkt.header.body_size.into()];
    if let Err(_e) = stream.try_read(&mut rcv_buf) {
        //eprintln!("{}", e);
        //return Err(e);
    }
    pkt.body = rcv_buf;

    return pkt;
}
