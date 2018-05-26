macro_rules! packet_payload {
    ($name:ident, op: $op:expr) => {
        impl PacketData for $name {
            const OPCODE: u32 = $op;
        }
    };
    ($name:ident<$life:tt>, op: $op:expr) => {
        impl<$life> PacketData for $name<$life> {
            const OPCODE: u32 = $op;
        }
    };
    ($name:ident, op: $op:expr, skip: true) => {
        impl PacketData for $name {
            const OPCODE: u32 = $op;
            fn skip_ser(&self) -> bool {
                true
            }
        }
        impl<'de> $crate::serde::de::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: $crate::serde::de::Deserializer<'de>,
            {
                deserializer.deserialize_ignored_any($crate::serde::de::IgnoredAny)?;
                Ok(Default::default())
            }
        }
    };
    ($name:ident, event: $evt:expr) => {
        impl PacketData for $name {
            const OPCODE: u32 = 0;
            const EVENT: Option<&'static str> = Some($evt);
        }
    };
    ($name:ident<$life:tt>, event: $evt:expr) => {
        impl<$life> PacketData for $name<$life> {
            const OPCODE: u32 = 0;
            const EVENT: Option<&'static str> = Some($evt);
        }
    };
}
