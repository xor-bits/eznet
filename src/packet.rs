use bitflags::bitflags;

bitflags! {
    pub struct PacketFlags: u8 {
        // 0 bit = padding
        // 1 bit = padding
        // 2 bit = padding
        // 3 bit = padding
        //
        // 4 bit = padding
        // 5 bit = ordering
        // 6 bit = reliability
        // 7 bit = reliability

        // packet reliability
        const UNRELIABLE           = 0b_0000_0000;
        const SEMI_RELIABLE        = 0b_0000_0001;
        const RELIABLE             = 0b_0000_0010;

        // packet ordering
        const UNORDERED            = 0b_0000_0000;
        const ORDERED              = 0b_0000_0100;

        // important packets = reliable
        const IMPORTANT_ORDERED    = Self::RELIABLE.bits | Self::ORDERED.bits;
        const IMPORTANT_UNORDERED  = Self::RELIABLE.bits | Self::UNORDERED.bits;
        const IMPORTANT            = Self::IMPORTANT_ORDERED.bits;

        // optional packets = unreliable
        const OPTIONAL_ORDERED     = Self::UNRELIABLE.bits | Self::ORDERED.bits;
        const OPTIONAL_UNORDERED   = Self::UNRELIABLE.bits | Self::UNORDERED.bits;
        const OPTIONAL             = Self::OPTIONAL_UNORDERED.bits;

        // desireable packets = semi reliable
        const DESIREABLE_ORDERED   = Self::SEMI_RELIABLE.bits | Self::ORDERED.bits;
        const DESIREABLE_UNORDERED = Self::SEMI_RELIABLE.bits | Self::UNORDERED.bits;
        const DESIREABLE           = Self::DESIREABLE_UNORDERED.bits;
    }
}
