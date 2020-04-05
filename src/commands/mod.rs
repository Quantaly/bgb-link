pub mod typed;

pub use typed::TypedBGBCommand;

pub trait BGBCommand {
    fn serialize(&self) -> [u8; 8];
}

#[derive(Debug, PartialEq)]
pub struct RawBGBCommand {
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
    pub b4: u8,
    pub i1: u32,
}

impl BGBCommand for RawBGBCommand {
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

impl RawBGBCommand {
    pub fn deserialize(bytes: &[u8; 8]) -> RawBGBCommand {
        let i1_bytes = [bytes[4], bytes[5], bytes[6], bytes[7]];
        RawBGBCommand {
            b1: bytes[0],
            b2: bytes[1],
            b3: bytes[2],
            b4: bytes[3],
            i1: u32::from_le_bytes(i1_bytes),
        }
    }
}

#[test]
fn raw_serialization() {
    assert_eq!(
        RawBGBCommand {
            b1: 1,
            b2: 2,
            b3: 3,
            b4: 4,
            i1: 5,
        }
        .serialize(),
        [1, 2, 3, 4, 5, 0, 0, 0]
    );

    assert_eq!(
        RawBGBCommand {
            b1: 5,
            b2: 4,
            b3: 3,
            b4: 2,
            i1: 1 << (8 * 3),
        }
        .serialize(),
        [5, 4, 3, 2, 0, 0, 0, 1]
    );
}

#[test]
fn raw_deserialization() {
    assert_eq!(
        RawBGBCommand::deserialize(&[1, 2, 3, 4, 5, 0, 0, 0]),
        RawBGBCommand {
            b1: 1,
            b2: 2,
            b3: 3,
            b4: 4,
            i1: 5,
        }
    );

    assert_eq!(
        RawBGBCommand::deserialize(&[5, 4, 3, 2, 0, 0, 0, 1]),
        RawBGBCommand {
            b1: 5,
            b2: 4,
            b3: 3,
            b4: 2,
            i1: 1 << (8 * 3),
        }
    );
}
