use bitflags::bitflags;
use serde::{Deserialize, Serialize};

//

bitflags! {
    pub struct PacketFlags: u8 {
        // packet reliability

        /// Packets may get dropped
        ///
        /// Fast and minimal overhead
        const UNRELIABLE           = 0b_0000;

        /// Packets may get dropped but less likely
        ///
        /// Fast but 2x bandwidth
        // const SEMI_RELIABLE        = 0b_0001;

        /// Packets are resent if dropped
        ///
        /// Slow but reliable
        const RELIABLE             = 0b_0010;

        // packet ordering

        /// Packets may be received out of order
        ///
        /// Fast and minimal overhead
        const UNORDERED            = 0b_0000;

        /// Packets out of order will be dropped
        ///
        /// Fast but less reliable
        const SEQUENCED            = 0b_0100;

        /// Packets are received in the same
        /// order as sent
        ///
        /// Slow but ordered
        const ORDERED              = 0b_1000;

        //

        /// Packets are received in order and
        /// reliably
        ///
        /// Example: game chat
        const PRESET_ASSERTIVE     = Self::RELIABLE.bits | Self::ORDERED.bits;

        /// Packets might be received out of
        /// order but will be received reliably
        ///
        /// Example: Server sending map pieces
        /// to the client
        const PRESET_IMPORTANT     = Self::RELIABLE.bits | Self::UNORDERED.bits;

        /// Old packets are discarded
        ///
        /// Example: game player movement inputs
        const PRESET_DEFAULT       = Self::UNRELIABLE.bits | Self::SEQUENCED.bits;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
    pub seq: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packet<T> {
    pub header: PacketHeader,
    pub payload: T,
}

impl<T> Packet<T> {
    pub fn new(seq: Option<u16>, payload: T) -> Self {
        Self {
            header: PacketHeader { seq },
            payload,
        }
    }
}

// 4 packets
//
// seq:0
// '4' + data
//
// seq:1
// data
//
// seq:2
// data
//
// seq:3
// data
/* pub struct Datagram {
    pub
} */
