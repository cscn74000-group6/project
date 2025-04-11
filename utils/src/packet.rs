use std::fmt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
//use tokio_stream::Stream;
// This is an enum that designates what flags we have
#[derive(Clone, Copy, PartialEq, Debug)]
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
            _ => {
                eprintln!("Invalid integer called for FlagState: {}", in_state);
                return FlagState::WARNING;
            }
        }
    }
}

// Struct for the Header in a packet
#[derive(Debug, Clone, PartialEq)]
pub struct PacketHeader {
    pub flag: FlagState,
    pub plane_id: u8,
    pub body_size: u16,
    pub seq_len: u8,
}

impl PacketHeader {
    /// Returns an "empty" PacketHeader, which is defined as flag: 0 (WARNING), plane_id: 0,
    /// and body_size: 0.
    pub fn init() -> PacketHeader {
        return PacketHeader {
            flag: FlagState::WARNING,
            plane_id: 0,
            body_size: 0,
            seq_len: 0,
        };
    }
    /// Deseralize_packet_header() takes in a u8 slice and returns an unpacked PacketHeader. The
    /// function deseralizes in the same way the serialize_packet_header works.
    pub fn deseralize_packet_header(stream: &[u8]) -> Result<PacketHeader, std::io::Error> {
        if stream.len() < 5 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Vector does not have enought elements for a PacketHeader",
            ));
        }
        let new_flag = stream.first().copied();
        let plane_id = stream.get(1).copied();
        let body_1 = stream.get(2).copied();
        let body_2 = stream.get(3).copied();
        let seq_len = stream.get(4).copied();

        match (new_flag, plane_id, body_1, body_2, seq_len) {
            (Some(new_flag), Some(plane_id), Some(body_1), Some(body_2), Some(seq_len)) => {
                Ok(PacketHeader {
                    flag: FlagState::init(new_flag),
                    plane_id,
                    body_size: u16::from_ne_bytes([body_1, body_2]),
                    seq_len,
                })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unable to parse PacketHeader fields from Vec<u8>",
            )),
        }
    }

    /// Serialize a packet header into a Vec<u8>
    pub fn seralize_packet_header(&self) -> Vec<u8> {
        let mut seralized_bytes: Vec<u8> = Vec::new();
        seralized_bytes.push(self.flag as u8);
        seralized_bytes.push(self.plane_id);
        seralized_bytes.extend_from_slice(&self.body_size.to_ne_bytes());
        seralized_bytes.push(self.seq_len);
        seralized_bytes
    }
}

#[derive(Debug, PartialEq)]
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
    pub fn seralize_packet_buf(&self) -> Vec<u8> {
        let mut seralized_bytes: Vec<u8> = Vec::new();
        seralized_bytes.extend(self.header.seralize_packet_header());
        seralized_bytes.extend_from_slice(&self.body);
        return seralized_bytes;
    }
}

/// Implementing the fmt::Display trait for FlagState so that it is compatible with println!
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

/// Implementing the fmt::Display trait for PacketHeader so that it is compatible with println!
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

/// Implementing the fmt::Display trait for Packet so that it is compatible with println!
impl fmt::Display for Packet {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match String::from_utf8(self.body.clone()) {
            Ok(body_string) => write!(f, "{0}\n{1}", self.header, body_string),
            Err(_) => write!(f, "{}\n{:?}", self.header, self.body.as_slice()),
        }
    }
}

/// To use the vector return, just use &[variablename]
pub async fn serialize_packet(pkt: Packet, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    stream.write_all(&pkt.seralize_packet_buf()).await
}

/// Takes in the TcpStream, reads values from it, and returns a Packet deseralized.
/// The logic works, except for the TcpStream which is still untested.
pub async fn deserialize_packet(stream: &mut TcpStream) -> Result<Packet, std::io::Error> {
    let mut rcv_buf_header: Vec<u8> = vec![0; 5];
    let mut pkt: Packet = Packet::init();
    stream.read_exact(&mut rcv_buf_header).await?;

    pkt.header = match PacketHeader::deseralize_packet_header(&rcv_buf_header) {
        Ok(header) => header,
        Err(e) => {
            return Err(e);
        }
    };

    let mut rcv_buf: Vec<u8> = vec![0; pkt.header.body_size.into()];
    stream.read_exact(&mut rcv_buf).await?;
    pkt.body = rcv_buf;

    Ok(pkt)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn test_FlagState_init() {
        let warning = FlagState::init(0);
        let collision = FlagState::init(1);
        let coordinate = FlagState::init(2);
        let exit = FlagState::init(3);
        let error = FlagState::init(4);
        assert_eq!(warning, FlagState::WARNING);
        assert_eq!(collision, FlagState::COLLISION);
        assert_eq!(coordinate, FlagState::COORDINATE);
        assert_eq!(exit, FlagState::EXIT);
        assert_eq!(error, FlagState::WARNING);
    }

    #[test]
    fn test_PacketHeader_init() {
        let expected = PacketHeader {
            flag: FlagState::WARNING,
            plane_id: 0,
            body_size: 0,
            seq_len: 0,
        };
        let actual = PacketHeader::init();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_seralizePacketHeader() {
        let expected = PacketHeader {
            flag: FlagState::COLLISION,
            plane_id: 2,
            body_size: 5,
            seq_len: 12,
        };

        let seralized = expected.seralize_packet_header();

        assert_eq!(expected.flag, FlagState::init(seralized[0]));
        assert_eq!(expected.plane_id, seralized[1]);
        assert_eq!(
            expected.body_size,
            u16::from_ne_bytes([seralized[2], seralized[3]])
        );
        assert_eq!(expected.seq_len, seralized[4]);
    }

    #[test]
    fn test_deseralizePacketHeader_success() {
        let expected = PacketHeader {
            flag: FlagState::COLLISION,
            plane_id: 2,
            body_size: 5,
            seq_len: 12,
        };

        let seralized = expected.seralize_packet_header();

        let actual = PacketHeader::deseralize_packet_header(&seralized);

        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_deseralizePacketHeader_lenLower5() {
        let expected = PacketHeader {
            flag: FlagState::COLLISION,
            plane_id: 2,
            body_size: 5,
            seq_len: 12,
        };

        let mut seralized = expected.seralize_packet_header();

        seralized.pop();
        let actual = PacketHeader::deseralize_packet_header(&seralized);

        let actualErrMsg = actual.unwrap_err().to_string();
        println!("{}", actualErrMsg);
        assert_eq!(
            actualErrMsg,
            "Vector does not have enought elements for a PacketHeader"
        );
    }

    #[test]
    fn test_Packet_init() {
        let actual = Packet {
            header: PacketHeader::init(),
            body: Vec::new(),
        };

        let expected = Packet::init();

        assert_eq!(actual, expected);
    }
}

//#[test]
//fn test_Packet_transmit() {
//    let bod: &[u8] = b"TRANSMISSION";
//    let expected = Packet {
//        header: PacketHeader {
//            seq_len: 1,
//            plane_id: 1,
//            flag: FlagState::COORDINATE,
//            body_size: bod.len().try_into().unwrap(),
//        },
//        body: bod.to_vec(),
//    };
//
//    println!("{}", expected);
//    let mut stream = tokio_stream::iter(&expected.seralize_packet_buf());
//    let actual = deserialize_packet(stream);
//    let actualPkt = actual.unwrap();
//
//    assert_eq!(actualPkt, expected)
//}
//#[test]
//fn test_deseralizePacketHeader_HeaderParseErr() {
//    let expected = PacketHeader {
//        flag: FlagState::COLLISION,
//        plane_id: 2,
//        body_size: 5,
//        seq_len: 12,
//    };
//
//    let mut seralized = expected.seralize_packet_header();
//
//    let actual = PacketHeader::deseralize_packet_header(&seralized);
//
//    let actualErrMsg = actual.unwrap_err().to_string();
//    println!("{}", actualErrMsg);
//    assert_eq!(
//        actualErrMsg,
//        "Unable to parse PacketHeader fields from Vec<u8>"
//    );
//}
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
