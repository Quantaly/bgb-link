use super::*;
use std::error::Error;
use std::fmt;
use TypedBgbCommand::*;

/// Particular commands and their relevant data.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedBgbCommand {
    Version {
        valid: bool,
    },
    Joypad {
        button_number: u8,
        pressed: bool,
    },
    Sync1 {
        data: u8,
        high_speed: bool,
        double_speed: bool,
        timestamp: u32,
    },
    Sync2 {
        data: u8,
    },
    Sync3Response,
    Sync3Timestamp {
        timestamp: u32,
    },
    Status {
        running: bool,
        paused: bool,
        support_reconnect: bool,
    },
    WantDisconnect,
}

impl TypedBgbCommand {
    /// Places the command data in the proper fields for serialization.
    pub fn to_raw(&self) -> RawBgbCommand {
        match *self {
            Version { valid } => {
                if valid {
                    RawBgbCommand {
                        b1: 1,
                        b2: 1,
                        b3: 4,
                        b4: 0,
                        i1: 0,
                    }
                } else {
                    RawBgbCommand {
                        b1: 1,
                        b2: 0,
                        b3: 0,
                        b4: 0,
                        i1: 0,
                    }
                }
            }
            Joypad {
                button_number,
                pressed,
            } => RawBgbCommand {
                b1: 101,
                b2: (button_number & 0b111) | (if pressed { 1 << 3 } else { 0 }),
                b3: 0,
                b4: 0,
                i1: 0,
            },
            Sync1 {
                data,
                high_speed,
                double_speed,
                timestamp,
            } => RawBgbCommand {
                b1: 104,
                b2: data,
                b3: 0b10000001
                    | (if high_speed { 1 << 1 } else { 0 })
                    | (if double_speed { 1 << 2 } else { 0 }),
                b4: 0,
                i1: timestamp,
            },
            Sync2 { data } => RawBgbCommand {
                b1: 105,
                b2: data,
                b3: 0x80,
                b4: 0,
                i1: 0,
            },
            Sync3Response => RawBgbCommand {
                b1: 106,
                b2: 1,
                b3: 0,
                b4: 0,
                i1: 0,
            },
            Sync3Timestamp { timestamp } => RawBgbCommand {
                b1: 106,
                b2: 0,
                b3: 0,
                b4: 0,
                i1: timestamp,
            },
            Status {
                running,
                paused,
                support_reconnect,
            } => RawBgbCommand {
                b1: 108,
                b2: (if running { 1 << 0 } else { 0 })
                    | (if paused { 1 << 1 } else { 0 })
                    | (if support_reconnect { 1 << 2 } else { 0 }),
                b3: 0,
                b4: 0,
                i1: 0,
            },
            WantDisconnect => RawBgbCommand {
                b1: 109,
                b2: 0,
                b3: 0,
                b4: 0,
                i1: 0,
            },
        }
    }

    /// Reads the command data from the raw fields.
    ///
    /// In most cases, this will accept malformed input and either ignore it or pass it along.
    /// The exceptions are if `b1` is not recognized as a valid command type or if the `b2`
    /// field of a `sync3` command is not recognized.
    pub fn from_raw(raw: &RawBgbCommand) -> Result<TypedBgbCommand, CommandError> {
        let RawBgbCommand { b1, b2, b3, b4, i1 } = *raw;
        match b1 {
            1 => Ok(Version {
                valid: (b2, b3, b4, i1) == (1, 4, 0, 0),
            }),
            101 => Ok(Joypad {
                button_number: b2 & 0b111,
                pressed: b2 & (1 << 3) > 0,
            }),
            104 => Ok(Sync1 {
                data: b2,
                high_speed: b3 & (1 << 1) > 0,
                double_speed: b3 & (1 << 2) > 0,
                timestamp: i1,
            }),
            105 => Ok(Sync2 { data: b2 }),
            106 => {
                if b2 == 1 {
                    Ok(Sync3Response)
                } else if b2 == 0 {
                    Ok(Sync3Timestamp { timestamp: i1 })
                } else {
                    Err(CommandError::new(String::from("invalid sync3 command")))
                }
            }
            108 => Ok(Status {
                running: b2 & (1 << 0) > 0,
                paused: b2 & (1 << 1) > 0,
                support_reconnect: b2 & (1 << 2) > 0,
            }),
            109 => Ok(WantDisconnect),
            _ => Err(CommandError::new(String::from("invalid command number"))),
        }
    }

    /// Read the provided buffer directly as a command.
    pub fn deserialize(bytes: &[u8; 8]) -> Result<TypedBgbCommand, CommandError> {
        TypedBgbCommand::from_raw(&RawBgbCommand::deserialize(bytes))
    }
}

impl BgbCommand for TypedBgbCommand {
    fn serialize(&self) -> [u8; 8] {
        self.to_raw().serialize()
    }
}

#[derive(Debug)]
pub struct CommandError {
    msg: String,
}

impl CommandError {
    fn new(msg: String) -> CommandError {
        CommandError { msg }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl Error for CommandError {}
