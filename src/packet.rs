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

/// If your data is static: do `Bytes::from(data)`
/// first to avoid copying or use the constructors
/// postfixed with `_static`.
///
/// TODO: specialization for
/// 'static when it is stable
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
    pub fn ordered<B: IntoBytes>(bytes: B, stream_id: Option<u8>) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Ordered { stream_id },
        }
    }

    /// no packets are dropped
    ///
    /// ordered
    pub fn ordered_static<B: IntoStaticBytes>(bytes: B, stream_id: Option<u8>) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Ordered { stream_id },
        }
    }

    /// old packets are dropped
    ///
    /// ordered
    pub fn reliable_sequenced<B: IntoBytes>(bytes: B, stream_id: Option<u8>) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::ReliableSequenced {
                stream_id,
                seq_id: 0,
            },
        }
    }

    /// old packets are dropped
    ///
    /// ordered
    pub fn reliable_sequenced_static<B: IntoStaticBytes>(bytes: B, stream_id: Option<u8>) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::ReliableSequenced {
                stream_id,
                seq_id: 0,
            },
        }
    }

    /// no packets are dropped
    ///
    /// not ordered
    pub fn reliable_unordered<B: IntoBytes>(bytes: B) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::ReliableUnordered,
        }
    }

    /// no packets are dropped
    ///
    /// not ordered
    pub fn reliable_unordered_static<B: IntoStaticBytes>(bytes: B) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::ReliableUnordered,
        }
    }

    /// 'random' and old packets are dropped
    ///
    /// ordered
    pub fn unreliable_sequenced<B: IntoBytes>(bytes: B, stream_id: Option<u8>) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::UnreliableSequenced {
                stream_id,
                seq_id: 0,
            },
        }
    }

    /// 'random' and old packets are dropped
    ///
    /// ordered
    pub fn unreliable_sequenced_static<B: IntoStaticBytes>(
        bytes: B,
        stream_id: Option<u8>,
    ) -> Self {
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
    pub fn unreliable<B: IntoBytes>(bytes: B) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Unreliable,
        }
    }

    /// 'random' packets are dropped
    ///
    /// not ordered
    pub fn unreliable_static<B: IntoStaticBytes>(bytes: B) -> Self {
        Self {
            bytes: bytes.into_bytes(),
            header: PacketHeader::Unreliable,
        }
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

impl IntoStaticBytes for &'static str {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}
