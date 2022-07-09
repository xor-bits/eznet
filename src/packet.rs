use bytes::Bytes;
use serde::{Deserialize, Serialize};

//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub enum PacketHeader {
    /// no packets are dropped
    ///
    /// ordered
    Ordered { stream_id: Option<u8> },

    /// old packets are dropped
    ///
    /// ordered
    ReliableSequenced { stream_id: Option<u8>, seq_id: u16 },

    /// no packets are dropped
    ///
    /// not ordered
    ReliableUnordered,

    /// 'random' and old packets are dropped
    ///
    /// ordered
    UnreliableSequenced { stream_id: Option<u8>, seq_id: u16 },

    /// 'random' packets are dropped
    ///
    /// not ordered
    Unreliable,
}

//

impl Default for PacketHeader {
    fn default() -> Self {
        Self::Ordered { stream_id: None }
    }
}

//

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet {
    pub header: PacketHeader,
    pub bytes: Bytes,
}

//

impl Packet {
    /// no packets are dropped
    ///
    /// ordered
    pub fn ordered(bytes: Bytes, stream: Option<u8>) -> Self {
        Self {
            bytes,
            header: PacketHeader::Ordered { stream_id: stream },
        }
    }

    /// old packets are dropped
    ///
    /// ordered
    pub fn reliable_sequenced(bytes: Bytes, stream: Option<u8>) -> Self {
        Self {
            bytes,
            header: PacketHeader::ReliableSequenced {
                stream_id: stream,
                seq_id: 0,
            },
        }
    }

    /// no packets are dropped
    ///
    /// not ordered
    pub fn reliable_unordered(bytes: Bytes) -> Self {
        Self {
            bytes,
            header: PacketHeader::ReliableUnordered,
        }
    }

    /// 'random' and old packets are dropped
    ///
    /// ordered
    pub fn unreliable_sequenced(bytes: Bytes, stream: Option<u8>) -> Self {
        Self {
            bytes,
            header: PacketHeader::UnreliableSequenced {
                stream_id: stream,
                seq_id: 0,
            },
        }
    }

    /// 'random' packets are dropped
    ///
    /// not ordered
    pub fn unreliable(bytes: Bytes) -> Self {
        Self {
            bytes,
            header: PacketHeader::Unreliable,
        }
    }
}
