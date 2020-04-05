use super::*;
use TypedBgbCommand::*;

#[derive(Debug, PartialEq)]
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

    pub fn from_raw(raw: &RawBgbCommand) -> Result<TypedBgbCommand, &'static str> {
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
                    Err("invalid sync3 command")
                }
            }
            108 => Ok(Status {
                running: b2 & (1 << 0) > 0,
                paused: b2 & (1 << 1) > 0,
                support_reconnect: b2 & (1 << 2) > 0,
            }),
            109 => Ok(WantDisconnect),
            _ => Err("invalid command number"),
        }
    }

    pub fn deserialize(bytes: &[u8; 8]) -> Result<TypedBgbCommand, &'static str> {
        TypedBgbCommand::from_raw(&RawBgbCommand::deserialize(bytes))
    }
}

impl BgbCommand for TypedBgbCommand {
    fn serialize(&self) -> [u8; 8] {
        self.to_raw().serialize()
    }
}

#[test]
fn typed_to_raw() {
    assert_eq!(
        Version { valid: true }.to_raw(),
        RawBgbCommand {
            b1: 1,
            b2: 1,
            b3: 4,
            b4: 0,
            i1: 0,
        }
    );

    // invalid version should have same b1 but different other fields
    {
        let actual = Version { valid: false }.to_raw();
        let valid = Version { valid: true }.to_raw();
        assert_eq!(actual.b1, valid.b1);
        assert_ne!(
            (actual.b2, actual.b3, actual.b4, actual.i1),
            (valid.b2, valid.b3, valid.b4, valid.i1)
        );
    }

    assert_eq!(
        Joypad {
            button_number: 5,
            pressed: true
        }
        .to_raw(),
        RawBgbCommand {
            b1: 101,
            b2: 0b1_101,
            b3: 0,
            b4: 0,
            i1: 0,
        }
    );

    assert_eq!(
        Joypad {
            button_number: 3,
            pressed: false
        }
        .to_raw(),
        RawBgbCommand {
            b1: 101,
            b2: 0b0_011,
            b3: 0,
            b4: 0,
            i1: 0,
        }
    );

    assert_eq!(
        Sync1 {
            data: 42,
            high_speed: true,
            double_speed: false,
            timestamp: 69420
        }
        .to_raw(),
        RawBgbCommand {
            b1: 104,
            b2: 42,
            b3: 0b10000011,
            b4: 0,
            i1: 69420,
        }
    );

    assert_eq!(
        Sync1 {
            data: 180,
            high_speed: false,
            double_speed: true,
            timestamp: 1234567890
        }
        .to_raw(),
        RawBgbCommand {
            b1: 104,
            b2: 180,
            b3: 0b10000101,
            b4: 0,
            i1: 1234567890,
        }
    );

    assert_eq!(
        Sync2 { data: 254 }.to_raw(),
        RawBgbCommand {
            b1: 105,
            b2: 254,
            b3: 0x80,
            b4: 0,
            i1: 0,
        }
    );

    // for sync3, b3 and b4 are deprecated
    // so don't test/guarantee them
    {
        let actual = Sync3Response.to_raw();
        let expected = RawBgbCommand {
            b1: 106,
            b2: 1,
            b3: 0,
            b4: 0,
            i1: 0,
        };
        assert_eq!(
            (actual.b1, actual.b2, actual.i1),
            (expected.b1, expected.b2, expected.i1)
        );

        let actual = Sync3Timestamp {
            timestamp: 88888888,
        }
        .to_raw();
        let expected = RawBgbCommand {
            b1: 106,
            b2: 0,
            b3: 0,
            b4: 0,
            i1: 88888888,
        };
        assert_eq!(
            (actual.b1, actual.b2, actual.i1),
            (expected.b1, expected.b2, expected.i1)
        );
    }

    assert_eq!(
        Status {
            running: true,
            paused: false,
            support_reconnect: false,
        }
        .to_raw(),
        RawBgbCommand {
            b1: 108,
            b2: 0x01,
            b3: 0,
            b4: 0,
            i1: 0,
        }
    );

    assert_eq!(
        Status {
            running: false,
            paused: true,
            support_reconnect: false,
        }
        .to_raw(),
        RawBgbCommand {
            b1: 108,
            b2: 0x02,
            b3: 0,
            b4: 0,
            i1: 0,
        }
    );

    assert_eq!(
        Status {
            running: false,
            paused: false,
            support_reconnect: true,
        }
        .to_raw(),
        RawBgbCommand {
            b1: 108,
            b2: 0x04,
            b3: 0,
            b4: 0,
            i1: 0,
        }
    );

    assert_eq!(
        WantDisconnect.to_raw(),
        RawBgbCommand {
            b1: 109,
            b2: 0,
            b3: 0,
            b4: 0,
            i1: 0,
        }
    );
}

#[test]
fn typed_from_raw() -> Result<(), &'static str> {
    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 1,
            b2: 1,
            b3: 4,
            b4: 0,
            i1: 0,
        })?,
        Version { valid: true }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 1,
            b2: 2,
            b3: 3,
            b4: 4,
            i1: 5,
        })?,
        Version { valid: false }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 101,
            b2: 0b1_101,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        Joypad {
            button_number: 5,
            pressed: true
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 101,
            b2: 0b0_011,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        Joypad {
            button_number: 3,
            pressed: false
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 104,
            b2: 42,
            b3: 0b10000011,
            b4: 0,
            i1: 69420,
        })?,
        Sync1 {
            data: 42,
            high_speed: true,
            double_speed: false,
            timestamp: 69420
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 104,
            b2: 180,
            b3: 0b10000101,
            b4: 0,
            i1: 1234567890,
        })?,
        Sync1 {
            data: 180,
            high_speed: false,
            double_speed: true,
            timestamp: 1234567890
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 105,
            b2: 254,
            b3: 0x80,
            b4: 0,
            i1: 0,
        })?,
        Sync2 { data: 254 }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 106,
            b2: 1,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        Sync3Response
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 106,
            b2: 0,
            b3: 0,
            b4: 0,
            i1: 88888888,
        })?,
        Sync3Timestamp {
            timestamp: 88888888,
        }
    );

    if let Err(msg) = TypedBgbCommand::from_raw(&RawBgbCommand {
        b1: 106,
        b2: 106,
        b3: 0,
        b4: 0,
        i1: 0,
    }) {
        assert!(msg.contains("sync3"));
    } else {
        panic!("no error for invalid sync3");
    }

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 108,
            b2: 0x01,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        Status {
            running: true,
            paused: false,
            support_reconnect: false,
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 108,
            b2: 0x02,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        Status {
            running: false,
            paused: true,
            support_reconnect: false,
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 108,
            b2: 0x04,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        Status {
            running: false,
            paused: false,
            support_reconnect: true,
        }
    );

    assert_eq!(
        TypedBgbCommand::from_raw(&RawBgbCommand {
            b1: 109,
            b2: 0,
            b3: 0,
            b4: 0,
            i1: 0,
        })?,
        WantDisconnect
    );

    if let Err(_) = TypedBgbCommand::from_raw(&RawBgbCommand {
        b1: 246,
        b2: 0,
        b3: 0,
        b4: 0,
        i1: 0,
    }) {
    } else {
        panic!("no error for invalid command number");
    }

    Ok(())
}
