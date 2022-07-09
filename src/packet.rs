use bytes::Bytes;

//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ReliableOrdered(Option<u16>),
    ReliableUnordered,
    Unreliable,
}

//

impl Default for PacketType {
    fn default() -> Self {
        Self::ReliableOrdered(None)
    }
}

//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub bytes: Bytes,
    pub ty: PacketType,
}

//

impl Packet {
    pub fn new(bytes: Bytes) -> Self {
        Self {
            bytes,
            ty: Default::default(),
        }
    }

    pub fn copy_from_slice(bytes: &[u8]) -> Self {
        Self::new(Bytes::copy_from_slice(bytes))
    }

    pub fn reliable_ordered(bytes: Bytes, stream: Option<u16>) -> Self {
        Self::new(bytes).as_reliable_ordered(stream)
    }

    pub fn reliable_unordered(bytes: Bytes) -> Self {
        Self::new(bytes).as_reliable_unordered()
    }

    pub fn unreliable(bytes: Bytes) -> Self {
        Self::new(bytes).as_unreliable()
    }

    pub fn with_type(mut self, ty: PacketType) -> Self {
        self.ty = ty;
        self
    }

    pub fn as_reliable_ordered(mut self, stream: Option<u16>) -> Self {
        self.ty = PacketType::ReliableOrdered(stream);
        self
    }

    pub fn as_reliable_unordered(mut self) -> Self {
        self.ty = PacketType::ReliableUnordered;
        self
    }

    pub fn as_unreliable(mut self) -> Self {
        self.ty = PacketType::Unreliable;
        self
    }
}

impl<T: Into<Bytes>> From<T> for Packet {
    fn from(bytes: T) -> Self {
        Self::new(bytes.into())
    }
}
