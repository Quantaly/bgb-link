mod tests;
pub mod typed;

pub use typed::TypedBgbCommand;

/// A common trait for anything that can be serialized into the BGB format.
pub trait BgbCommand {
    /// Serializes the object into an 8-byte packet.
    fn serialize(&self) -> [u8; 8];
}

/// Contains the raw structure of a BGB command.
#[derive(Debug, PartialEq)]
pub struct RawBgbCommand {
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
    pub b4: u8,
    pub i1: u32,
}

impl BgbCommand for RawBgbCommand {
    fn serialize(&self) -> [u8; 8] {
        let i1_bytes = self.i1.to_le_bytes();
        [
            self.b1,
            self.b2,
            self.b3,
            self.b4,
            i1_bytes[0],
            i1_bytes[1],
            i1_bytes[2],
            i1_bytes[3],
        ]
    }
}

impl RawBgbCommand {
    /// Reads the fields from a serialized BGB command.
    pub fn deserialize(bytes: &[u8; 8]) -> RawBgbCommand {
        let i1_bytes = [bytes[4], bytes[5], bytes[6], bytes[7]];
        RawBgbCommand {
            b1: bytes[0],
            b2: bytes[1],
            b3: bytes[2],
            b4: bytes[3],
            i1: u32::from_le_bytes(i1_bytes),
        }
    }
}
