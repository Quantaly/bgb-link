#[test]
fn raw_serialization() {
    use super::*;

    assert_eq!(
        RawBgbCommand {
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
        RawBgbCommand {
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
    use super::*;
    assert_eq!(
        RawBgbCommand::deserialize(&[1, 2, 3, 4, 5, 0, 0, 0]),
        RawBgbCommand {
            b1: 1,
            b2: 2,
            b3: 3,
            b4: 4,
            i1: 5,
        }
    );

    assert_eq!(
        RawBgbCommand::deserialize(&[5, 4, 3, 2, 0, 0, 0, 1]),
        RawBgbCommand {
            b1: 5,
            b2: 4,
            b3: 3,
            b4: 2,
            i1: 1 << (8 * 3),
        }
    );
}

#[test]
fn typed_to_raw() {
    use super::typed::TypedBgbCommand::*;
    use super::*;

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
fn typed_from_raw() -> Result<(), super::typed::CommandError> {
    use super::typed::TypedBgbCommand::*;
    use super::*;

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

    if let Err(e) = TypedBgbCommand::from_raw(&RawBgbCommand {
        b1: 106,
        b2: 106,
        b3: 0,
        b4: 0,
        i1: 0,
    }) {
        assert!(format!("{}", e).contains("sync3"));
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
