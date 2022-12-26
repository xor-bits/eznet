use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub enum PacketHeader {
    /// no packets are dropped
    ///
    /// ordered
    Ordered { stream_id: u8 },

    /// no packets are dropped
    ///
    /// not ordered
    Unordered,

    /// old packets are dropped
    ///
    /// ordered
    Sequenced { stream_id: u8, seq_id: u16 },

    /// 'random' and old packets are dropped
    ///
    /// ordered
    UnreliableSequenced { stream_id: u8, seq_id: u16 },

    /// 'random' packets are dropped
    ///
    /// not ordered
    UnreliableUnordered,
}

//

impl Default for PacketHeader {
    fn default() -> Self {
        Self::Ordered { stream_id: 0 }
    }
}

//

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet {
    pub header: PacketHeader,
    pub bytes: Bytes,
}

//

impl PacketHeader {
    pub fn with_seq_id(self, seq_id: u16) -> Self {
        match self {
            PacketHeader::Sequenced { stream_id, .. } => {
                PacketHeader::Sequenced { stream_id, seq_id }
            }
            PacketHeader::UnreliableSequenced { stream_id, .. } => {
                Self::UnreliableSequenced { stream_id, seq_id }
            }
            other => other,
        }
    }
}

impl Packet {
    /// no packets are dropped
    ///
    /// ordered
    pub fn ordered<B: IntoBytes>(bytes: B, stream_id: u8) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Ordered { stream_id },
        }
    }

    /// no packets are dropped
    ///
    /// not ordered
    pub fn unordered<B: IntoBytes>(bytes: B) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Unordered,
        }
    }

    /// old packets are dropped
    ///
    /// ordered
    pub fn sequenced<B: IntoBytes>(bytes: B, stream_id: u8) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Sequenced {
                stream_id,
                seq_id: 0,
            },
        }
    }

    /// 'random' and old packets are dropped
    ///
    /// ordered
    pub fn unreliable_sequenced<B: IntoBytes>(bytes: B, stream_id: u8) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::UnreliableSequenced {
                stream_id,
                seq_id: 0,
            },
        }
    }

    /// 'random' packets are dropped
    ///
    /// not ordered
    pub fn unreliable_unordered<B: IntoBytes>(bytes: B) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::UnreliableUnordered,
        }
    }

    pub fn with_seq_id(self, seq_id: u16) -> Self {
        Self {
            bytes: self.bytes,
            header: self.header.with_seq_id(seq_id),
        }
    }

    pub fn encode(&self) -> Result<Bytes, bincode::Error> {
        let mut packet = BytesMut::new().writer();
        bincode::serialize_into(&mut packet, self)?;
        Ok(packet.into_inner().into())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

//

pub trait IntoBytes {
    fn into_bytes(self) -> Bytes;
}

pub trait IntoStaticBytes {
    fn into_bytes(self) -> Bytes;
}

impl IntoBytes for Bytes {
    fn into_bytes(self) -> Bytes {
        self
    }
}

impl IntoBytes for Vec<u8> {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}

impl IntoBytes for Box<[u8]> {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}

impl IntoBytes for String {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}

impl<'a> IntoBytes for &'a [u8] {
    fn into_bytes(self) -> Bytes {
        // TODO: specialization for
        // 'static when it is stable
        Bytes::copy_from_slice(self)
    }
}

impl<'a, const C: usize> IntoBytes for &'a [u8; C] {
    fn into_bytes(self) -> Bytes {
        // TODO: specialization for
        // 'static when it is stable
        Bytes::copy_from_slice(self)
    }
}

impl<const C: usize> IntoBytes for [u8; C] {
    fn into_bytes(self) -> Bytes {
        // TODO: specialization for
        // 'static when it is stable
        Bytes::copy_from_slice(&self)
    }
}

impl<'a> IntoBytes for &'a str {
    fn into_bytes(self) -> Bytes {
        // TODO: specialization for
        // 'static when it is stable
        Bytes::from(self.to_owned())
    }
}

// TODO: specialization for
// T: Serialize when it is stable

impl IntoStaticBytes for &'static [u8] {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}

impl<const C: usize> IntoStaticBytes for &'static [u8; C] {
    fn into_bytes(self) -> Bytes {
        self[..].into()
    }
}

impl IntoStaticBytes for &'static str {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}
