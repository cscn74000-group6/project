use std::fmt;
use tokio::net::TcpStream;

// This is an enum that designates what flags we have
#[derive(Clone, Copy)]
pub enum FlagState {
    WARNING = 0,
    COLLISION = 1,
    COORDINATE = 2,
    EXIT = 3,
}

impl FlagState {
    // init takes a u8 in to give back a FlagState. Prints out a warning but returns the default
    // value of WARNING if it doesn't match any of the values in the enum
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

// Struct for the Header in a packet
pub struct PacketHeader {
    pub flag: FlagState,
    pub plane_id: u8,
    pub body_size: u16,
}

impl PacketHeader {
    // init() returns an "empty" PacketHeader, which is defined as flag: 0 (WARNING), plane_id: 0,
    // and body_size: 0.
    pub fn init() -> PacketHeader {
        return PacketHeader {
            flag: FlagState::WARNING,
            plane_id: 0,
            body_size: 0,
        };
    }
    // deseralize_packet_header() takes in a vector<u8> and returns an unpacked PacketHeader. The
    // function deseralizes in the same way the serialize_packet_header works.
    pub fn deseralize_packet_header(stream: Vec<u8>) -> PacketHeader {
        return PacketHeader {
            flag: FlagState::init(*stream.get(0).unwrap()),
            plane_id: *stream.get(1).unwrap(),
            body_size: u16::from_ne_bytes([(stream[2]), (stream[3])]),
        };
    }

    pub fn seralize_packet_header(&self) -> Vec<u8> {
        let mut seralized_bytes: Vec<u8> = Vec::new();
        seralized_bytes.push(self.flag as u8);
        seralized_bytes.push(self.plane_id);
        seralized_bytes.extend_from_slice(&self.body_size.to_ne_bytes());
        return seralized_bytes;
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

// Implementing the fmt::Display trait for FlagState so that it is compatible with println!
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

// Implementing the fmt::Display trait for PacketHeader so that it is compatible with println!
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

// Implementing the fmt::Display trait for Packet so that it is compatible with println!
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
    seralized_bytes.extend(pkt.header.seralize_packet_header());
    seralized_bytes.extend_from_slice(&pkt.body);
    return seralized_bytes;
}

// Takes in the TcpStream, reads values from it, and returns a Packet deseralized.
// The logic works, except for the TcpStream which is still untested.
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

//fn unitTest() {
//        let bod: &[u8] = b"TRANSMISSION";
//    let send_pkt: Packet = Packet {
//        header: PacketHeader {
//            plane_id: 1,
//            flag: packet::FlagState::COORDINATE,
//            body_size: bod.len().try_into().unwrap(),
//        },
//        body: bod.to_vec(),
//    };
//
//    println!("{}", send_pkt);
//
//    let transmit: Vec<u8> = serialize_packet(send_pkt);
//
//    let mut pkt: Packet = Packet::init();
//
//    let rcv_buf_header = transmit[0..std::mem::size_of::<PacketHeader>()].to_vec();
//    pkt.header = PacketHeader::deseralize_packet_header(rcv_buf_header);
//
//    let rcv_buf: Vec<u8> = transmit[std::mem::size_of::<PacketHeader>()..].to_vec();
//    pkt.body = rcv_buf;
//
//    println!("{}", pkt);
//
//}
