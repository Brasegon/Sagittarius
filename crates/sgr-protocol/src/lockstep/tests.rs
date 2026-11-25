//! Unit tests of the lockstep frames, checked against the reference frames of `docs/icd.md`.

use super::*;

// Reference frames, copied from the "Trames de référence" table of docs/icd.md.
// If the ICD changes, these change with it.

/// tick 1, BAT_V 28 000 mV, BAT_TEMP -5.25 °C
const SENSOR_REF_A: [u8; 10] = [0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x6D, 0x60, 0xFD, 0xF3];
/// tick 74 565, BAT_V 33 600 mV, BAT_TEMP 21.50 °C
const SENSOR_REF_B: [u8; 10] = [0x01, 0x01, 0x00, 0x01, 0x23, 0x45, 0x83, 0x40, 0x08, 0x66];
/// tick 1, BAT_HEATER on
const ACTUATOR_REF_A: [u8; 7] = [0x01, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01];
/// tick 74 565, everything off
const ACTUATOR_REF_B: [u8; 7] = [0x01, 0x02, 0x00, 0x01, 0x23, 0x45, 0x00];

/// Builds a `SensorFrame` from flat values, so that each test case fits on one line.
const fn sensor_frame(tick: u32, bat_v: u16, bat_temp: i16) -> SensorFrame {
    SensorFrame {
        tick,
        power: PowerSensors { bat_v },
        thermal: ThermalSensors { bat_temp },
    }
}

const SENSOR_A: SensorFrame = sensor_frame(1, 28_000, -525);
const SENSOR_B: SensorFrame = sensor_frame(74_565, 33_600, 2_150);
const ACTUATOR_A: ActuatorFrame = ActuatorFrame {
    tick: 1,
    switches: SWITCH_BAT_HEATER,
};
const ACTUATOR_B: ActuatorFrame = ActuatorFrame {
    tick: 74_565,
    switches: 0,
};

#[test]
fn constants_match_the_icd() {
    assert_eq!(LOCKSTEP_VERSION, 1);
    assert_eq!(SWITCH_BAT_HEATER, 0b0000_0001);
    assert_eq!(SensorFrame::LEN, 10);
    assert_eq!(ActuatorFrame::LEN, 7);
}

// --- SensorFrame ---

#[test]
fn sensor_frame_encodes_reference_frames() {
    assert_eq!(SENSOR_A.encode(), SENSOR_REF_A);
    assert_eq!(SENSOR_B.encode(), SENSOR_REF_B);
}

#[test]
fn sensor_frame_decodes_reference_frames() {
    assert_eq!(SensorFrame::decode(&SENSOR_REF_A), Ok(SENSOR_A));
    assert_eq!(SensorFrame::decode(&SENSOR_REF_B), Ok(SENSOR_B));
}

#[test]
fn sensor_frame_round_trips_extreme_values() {
    for frame in [
        sensor_frame(0, 0, 0),
        sensor_frame(u32::MAX, u16::MAX, i16::MIN),
        sensor_frame(u32::MAX, u16::MAX, i16::MAX),
    ] {
        assert_eq!(SensorFrame::decode(&frame.encode()), Ok(frame));
    }
}

#[test]
fn sensor_frame_rejects_wrong_length() {
    // Empty, one byte short, one byte long.
    assert_eq!(SensorFrame::decode(&[]), Err(DecodeError::WrongLength));
    assert_eq!(
        SensorFrame::decode(&SENSOR_REF_A[..9]),
        Err(DecodeError::WrongLength)
    );
    let mut too_long = [0u8; 11];
    too_long[..10].copy_from_slice(&SENSOR_REF_A);
    assert_eq!(
        SensorFrame::decode(&too_long),
        Err(DecodeError::WrongLength)
    );
}

#[test]
fn sensor_frame_rejects_unsupported_version() {
    for version in [0, 2, 0xFF] {
        let mut bytes = SENSOR_REF_A;
        bytes[0] = version;
        assert_eq!(
            SensorFrame::decode(&bytes),
            Err(DecodeError::UnsupportedVersion)
        );
    }
}

#[test]
fn sensor_frame_rejects_unexpected_frame_type() {
    // 2 is a known type, but it belongs to the other direction.
    for frame_type in [0, 2, 0xFF] {
        let mut bytes = SENSOR_REF_A;
        bytes[1] = frame_type;
        assert_eq!(
            SensorFrame::decode(&bytes),
            Err(DecodeError::UnexpectedFrameType)
        );
    }
}

// --- ActuatorFrame ---

#[test]
fn actuator_frame_encodes_reference_frames() {
    assert_eq!(ACTUATOR_A.encode(), ACTUATOR_REF_A);
    assert_eq!(ACTUATOR_B.encode(), ACTUATOR_REF_B);
}

#[test]
fn actuator_frame_decodes_reference_frames() {
    assert_eq!(ActuatorFrame::decode(&ACTUATOR_REF_A), Ok(ACTUATOR_A));
    assert_eq!(ActuatorFrame::decode(&ACTUATOR_REF_B), Ok(ACTUATOR_B));
}

#[test]
fn actuator_frame_round_trips_extreme_values() {
    for frame in [
        ActuatorFrame {
            tick: 0,
            switches: 0,
        },
        ActuatorFrame {
            tick: u32::MAX,
            switches: SWITCH_BAT_HEATER,
        },
    ] {
        assert_eq!(ActuatorFrame::decode(&frame.encode()), Ok(frame));
    }
}

#[test]
fn actuator_frame_rejects_wrong_length() {
    // Empty, one byte short, one byte long.
    assert_eq!(ActuatorFrame::decode(&[]), Err(DecodeError::WrongLength));
    assert_eq!(
        ActuatorFrame::decode(&ACTUATOR_REF_A[..6]),
        Err(DecodeError::WrongLength)
    );
    let mut too_long = [0u8; 8];
    too_long[..7].copy_from_slice(&ACTUATOR_REF_A);
    assert_eq!(
        ActuatorFrame::decode(&too_long),
        Err(DecodeError::WrongLength)
    );
}

#[test]
fn actuator_frame_rejects_unsupported_version() {
    for version in [0, 2, 0xFF] {
        let mut bytes = ACTUATOR_REF_A;
        bytes[0] = version;
        assert_eq!(
            ActuatorFrame::decode(&bytes),
            Err(DecodeError::UnsupportedVersion)
        );
    }
}

#[test]
fn actuator_frame_rejects_unexpected_frame_type() {
    // 1 is a known type, but it belongs to the other direction.
    for frame_type in [0, 1, 0xFF] {
        let mut bytes = ACTUATOR_REF_A;
        bytes[1] = frame_type;
        assert_eq!(
            ActuatorFrame::decode(&bytes),
            Err(DecodeError::UnexpectedFrameType)
        );
    }
}

#[test]
fn actuator_frame_ignores_reserved_bits() {
    // ICD: reserved bits are ignored on reception, so the frame stays valid.
    let mut bytes = ACTUATOR_REF_A;

    bytes[6] = 0b1111_1111;
    let frame = ActuatorFrame::decode(&bytes).unwrap();
    assert_ne!(frame.switches & SWITCH_BAT_HEATER, 0);

    bytes[6] = 0b1111_1110;
    let frame = ActuatorFrame::decode(&bytes).unwrap();
    assert_eq!(frame.switches & SWITCH_BAT_HEATER, 0);
}

// --- Both ---

#[test]
fn all_zero_buffer_is_not_a_frame() {
    assert!(SensorFrame::decode(&[0; SensorFrame::LEN]).is_err());
    assert!(ActuatorFrame::decode(&[0; ActuatorFrame::LEN]).is_err());
}
