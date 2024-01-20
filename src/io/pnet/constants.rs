macro_rules! constants {
    ($name:ident {$($field_name:ident = $value:literal,)*}) => {
        #[repr(u32)]
        pub enum $name {
            $($field_name = $value,)*
        }

        impl std::ops::Add<u32> for $name {
            type Output = u32;

            fn add(self, rhs: u32) -> Self::Output {
                self as u32 + rhs
            }
        }

        impl std::ops::Add<$name> for u32 {
            type Output = u32;

            fn add(self, rhs: $name) -> Self::Output {
                self + rhs as u32
            }
        }
    };
}

constants!(C2SSequenceNumbers {
    // Standard SLP
    SlpSyn = 0x00000000,
    SlpAck = 0x00000001,

    // Legacy
    LegacySyn = 0x10000000,
    LegacyAck = 0x10000001,
});

constants!(C2SAcknowledgementNumbers {
    // Standard SLP
    SlpAck = 0x00000001,

    // Legacy
    LegacyAck = 0x10000001,
});

constants!(S2CSequenceNumbers {
    // Standard SLP
    SlpSynAck = 0x00000000,

    // Legacy
    LegacySynAck = 0x10000000,
});

constants!(S2CAcknowledgementNumbers {
    // Standard SLP
    SlpSynAck = 0x00000001,

    // Legacy
    LegacySynAck = 0x10000001,
});
