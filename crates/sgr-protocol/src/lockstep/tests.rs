//! Unit tests of the lockstep frames, checked against the reference frames of `docs/icd.md`.

use super::*;

// Reference frames, copied from the "Trames de référence" table of docs/icd.md.
// If the ICD changes, these change with it.

/// `Tick`, tick 1
const TICK_REF: [u8; 6] = [0x02, 0x01, 0x00, 0x00, 0x00, 0x01];
/// `CycleDone`, tick 1
const CYCLE_DONE_REF: [u8; 6] = [0x02, 0x04, 0x00, 0x00, 0x00, 0x01];

#[test]
fn constants_match_the_icd() {
    assert_eq!(LOCKSTEP_VERSION, 2);
    assert_eq!(Tick::LEN, 6);
    assert_eq!(CycleDone::LEN, 6);
}

// --- Tick ---

#[test]
fn tick_encodes_reference_frame() {
    assert_eq!(Tick { tick: 1 }.encode(), TICK_REF);
}

#[test]
fn tick_decodes_reference_frame() {
    assert_eq!(Tick::decode(&TICK_REF), Ok(Tick { tick: 1 }));
}

#[test]
fn tick_puts_the_tick_in_big_endian() {
    // 74 565 is 0x00012345: the most significant byte comes first.
    let bytes = Tick { tick: 74_565 }.encode();
    assert_eq!(bytes, [0x02, 0x01, 0x00, 0x01, 0x23, 0x45]);
}

#[test]
fn tick_round_trips_extreme_values() {
    for frame in [Tick { tick: 0 }, Tick { tick: u32::MAX }] {
        assert_eq!(Tick::decode(&frame.encode()), Ok(frame));
    }
}

#[test]
fn tick_rejects_wrong_length() {
    // Empty, one byte short, one byte long.
    assert_eq!(Tick::decode(&[]), Err(DecodeError::WrongLength));
    assert_eq!(Tick::decode(&TICK_REF[..5]), Err(DecodeError::WrongLength));
    let mut too_long = [0u8; 7];
    too_long[..6].copy_from_slice(&TICK_REF);
    assert_eq!(Tick::decode(&too_long), Err(DecodeError::WrongLength));
}

#[test]
fn tick_rejects_unsupported_version() {
    // 1 was the version of the ICD v0.1 frames.
    for version in [0, 1, 3, 0xFF] {
        let mut bytes = TICK_REF;
        bytes[0] = version;
        assert_eq!(Tick::decode(&bytes), Err(DecodeError::UnsupportedVersion));
    }
}

#[test]
fn tick_rejects_unexpected_frame_type() {
    // 2, 3 and 4 are known types, but they are not a `Tick`.
    for frame_type in [0, 2, 3, 4, 0xFF] {
        let mut bytes = TICK_REF;
        bytes[1] = frame_type;
        assert_eq!(Tick::decode(&bytes), Err(DecodeError::UnexpectedFrameType));
    }
}

// --- CycleDone ---

#[test]
fn cycle_done_encodes_reference_frame() {
    assert_eq!(CycleDone { tick: 1 }.encode(), CYCLE_DONE_REF);
}

#[test]
fn cycle_done_decodes_reference_frame() {
    assert_eq!(
        CycleDone::decode(&CYCLE_DONE_REF),
        Ok(CycleDone { tick: 1 })
    );
}

#[test]
fn cycle_done_round_trips_extreme_values() {
    for frame in [CycleDone { tick: 0 }, CycleDone { tick: u32::MAX }] {
        assert_eq!(CycleDone::decode(&frame.encode()), Ok(frame));
    }
}

#[test]
fn cycle_done_rejects_wrong_length() {
    // Empty, one byte short, one byte long.
    assert_eq!(CycleDone::decode(&[]), Err(DecodeError::WrongLength));
    assert_eq!(
        CycleDone::decode(&CYCLE_DONE_REF[..5]),
        Err(DecodeError::WrongLength)
    );
    let mut too_long = [0u8; 7];
    too_long[..6].copy_from_slice(&CYCLE_DONE_REF);
    assert_eq!(CycleDone::decode(&too_long), Err(DecodeError::WrongLength));
}

#[test]
fn cycle_done_rejects_unsupported_version() {
    for version in [0, 1, 3, 0xFF] {
        let mut bytes = CYCLE_DONE_REF;
        bytes[0] = version;
        assert_eq!(
            CycleDone::decode(&bytes),
            Err(DecodeError::UnsupportedVersion)
        );
    }
}

#[test]
fn cycle_done_rejects_unexpected_frame_type() {
    // 1, 2 and 3 are known types, but they are not a `CycleDone`.
    for frame_type in [0, 1, 2, 3, 0xFF] {
        let mut bytes = CYCLE_DONE_REF;
        bytes[1] = frame_type;
        assert_eq!(
            CycleDone::decode(&bytes),
            Err(DecodeError::UnexpectedFrameType)
        );
    }
}

// --- BusRequest ---

/// `BusRequest`, tick 1, to the PCDU (0x10), payload `01` (GET_HK)
const GET_HK_REQUEST_REF: [u8; 8] = [0x02, 0x02, 0x00, 0x00, 0x00, 0x01, 0x10, 0x01];
/// `BusRequest`, tick 1, to the PCDU (0x10), payload `02 01` (SET_SWITCHES, BAT_HEATER closed)
const SET_SWITCHES_REQUEST_REF: [u8; 9] = [0x02, 0x02, 0x00, 0x00, 0x00, 0x01, 0x10, 0x02, 0x01];

/// Builds a `BusRequest` from a plain slice, so that each test case fits on one line.
fn bus_request(tick: u32, address: u8, payload: &[u8]) -> BusRequest {
    BusRequest {
        tick,
        address,
        payload: Payload::from_slice(payload).unwrap(),
    }
}

#[test]
fn bus_constants_match_the_icd() {
    assert_eq!(MAX_PAYLOAD, 32);
    assert_eq!(MAX_FRAME_LEN, 39);
}

#[test]
fn bus_request_encodes_reference_frames() {
    assert_eq!(
        &bus_request(1, 0x10, &[0x01]).encode()[..],
        &GET_HK_REQUEST_REF[..]
    );
    assert_eq!(
        &bus_request(1, 0x10, &[0x02, 0x01]).encode()[..],
        &SET_SWITCHES_REQUEST_REF[..]
    );
}

#[test]
fn bus_request_decodes_reference_frames() {
    assert_eq!(
        BusRequest::decode(&GET_HK_REQUEST_REF),
        Ok(bus_request(1, 0x10, &[0x01]))
    );
    assert_eq!(
        BusRequest::decode(&SET_SWITCHES_REQUEST_REF),
        Ok(bus_request(1, 0x10, &[0x02, 0x01]))
    );
}

#[test]
fn bus_request_accepts_an_empty_payload() {
    let frame = bus_request(74_565, 0x7F, &[]);
    let bytes = frame.encode();
    assert_eq!(&bytes[..], &[0x02, 0x02, 0x00, 0x01, 0x23, 0x45, 0x7F][..]);
    assert_eq!(BusRequest::decode(&bytes), Ok(frame));
}

#[test]
fn bus_request_accepts_the_longest_payload() {
    // 32 different bytes, to catch a payload that would be truncated or shifted.
    let payload: [u8; MAX_PAYLOAD] = core::array::from_fn(|i| 0xA0 + i as u8);
    let frame = bus_request(u32::MAX, 0xFF, &payload);
    let bytes = frame.encode();
    assert_eq!(bytes.len(), MAX_FRAME_LEN);
    assert_eq!(&bytes[7..], &payload[..]);
    assert_eq!(BusRequest::decode(&bytes), Ok(frame));
}

#[test]
fn bus_request_rejects_wrong_length() {
    // Empty, one byte short of the 7 fixed bytes, one byte past the longest frame.
    assert_eq!(BusRequest::decode(&[]), Err(DecodeError::WrongLength));
    assert_eq!(
        BusRequest::decode(&GET_HK_REQUEST_REF[..6]),
        Err(DecodeError::WrongLength)
    );
    let mut too_long = [0u8; MAX_FRAME_LEN + 1];
    too_long[..8].copy_from_slice(&GET_HK_REQUEST_REF);
    assert_eq!(BusRequest::decode(&too_long), Err(DecodeError::WrongLength));
}

#[test]
fn bus_request_rejects_unsupported_version() {
    for version in [0, 1, 3, 0xFF] {
        let mut bytes = GET_HK_REQUEST_REF;
        bytes[0] = version;
        assert_eq!(
            BusRequest::decode(&bytes),
            Err(DecodeError::UnsupportedVersion)
        );
    }
}

#[test]
fn bus_request_rejects_unexpected_frame_type() {
    // 1, 3 and 4 are known types, but they are not a `BusRequest`.
    for frame_type in [0, 1, 3, 4, 0xFF] {
        let mut bytes = GET_HK_REQUEST_REF;
        bytes[1] = frame_type;
        assert_eq!(
            BusRequest::decode(&bytes),
            Err(DecodeError::UnexpectedFrameType)
        );
    }
}

// --- BusResponse ---

/// `BusResponse`, tick 1, OK, PCDU housekeeping: 28 000 mV, -5.25 °C, every switch open
const HK_RESPONSE_REF: [u8; 12] = [
    0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x6D, 0x60, 0xFD, 0xF3, 0x00,
];
/// `BusResponse`, tick 1, OK, empty payload
const EMPTY_RESPONSE_REF: [u8; 7] = [0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00];
/// `BusResponse`, tick 74 565, OK, PCDU housekeeping: 33 600 mV, 21.50 °C, BAT_HEATER closed
const HK_RESPONSE_B_REF: [u8; 12] = [
    0x02, 0x03, 0x00, 0x01, 0x23, 0x45, 0x00, 0x83, 0x40, 0x08, 0x66, 0x01,
];
/// `BusResponse`, tick 74 565, NO_RESPONSE
const NO_RESPONSE_REF: [u8; 7] = [0x02, 0x03, 0x00, 0x01, 0x23, 0x45, 0x01];

/// Builds a `BusResponse` from a plain slice, so that each test case fits on one line.
fn bus_response(tick: u32, status: BusStatus, payload: &[u8]) -> BusResponse {
    BusResponse {
        tick,
        status,
        payload: Payload::from_slice(payload).unwrap(),
    }
}

/// The four reference frames, each with the value it stands for.
fn bus_response_references() -> [(&'static [u8], BusResponse); 4] {
    [
        (
            &HK_RESPONSE_REF,
            bus_response(1, BusStatus::Ok, &[0x6D, 0x60, 0xFD, 0xF3, 0x00]),
        ),
        (&EMPTY_RESPONSE_REF, bus_response(1, BusStatus::Ok, &[])),
        (
            &HK_RESPONSE_B_REF,
            bus_response(74_565, BusStatus::Ok, &[0x83, 0x40, 0x08, 0x66, 0x01]),
        ),
        (
            &NO_RESPONSE_REF,
            bus_response(74_565, BusStatus::NoResponse, &[]),
        ),
    ]
}

#[test]
fn bus_response_encodes_reference_frames() {
    for (bytes, frame) in bus_response_references() {
        assert_eq!(&frame.encode()[..], bytes);
    }
}

#[test]
fn bus_response_decodes_reference_frames() {
    for (bytes, frame) in bus_response_references() {
        assert_eq!(BusResponse::decode(bytes), Ok(frame));
    }
}

#[test]
fn bus_response_accepts_the_longest_payload() {
    // 32 different bytes, to catch a payload that would be truncated or shifted.
    let payload: [u8; MAX_PAYLOAD] = core::array::from_fn(|i| 0xA0 + i as u8);
    let frame = bus_response(u32::MAX, BusStatus::Ok, &payload);
    let bytes = frame.encode();
    assert_eq!(bytes.len(), MAX_FRAME_LEN);
    assert_eq!(&bytes[7..], &payload[..]);
    assert_eq!(BusResponse::decode(&bytes), Ok(frame));
}

#[test]
fn bus_response_rejects_wrong_length() {
    // Empty, one byte short of the 7 fixed bytes, one byte past the longest frame.
    assert_eq!(BusResponse::decode(&[]), Err(DecodeError::WrongLength));
    assert_eq!(
        BusResponse::decode(&EMPTY_RESPONSE_REF[..6]),
        Err(DecodeError::WrongLength)
    );
    let mut too_long = [0u8; MAX_FRAME_LEN + 1];
    too_long[..7].copy_from_slice(&EMPTY_RESPONSE_REF);
    assert_eq!(
        BusResponse::decode(&too_long),
        Err(DecodeError::WrongLength)
    );
}

#[test]
fn bus_response_rejects_unsupported_version() {
    for version in [0, 1, 3, 0xFF] {
        let mut bytes = HK_RESPONSE_REF;
        bytes[0] = version;
        assert_eq!(
            BusResponse::decode(&bytes),
            Err(DecodeError::UnsupportedVersion)
        );
    }
}

#[test]
fn bus_response_rejects_unexpected_frame_type() {
    // 1, 2 and 4 are known types, but they are not a `BusResponse`.
    for frame_type in [0, 1, 2, 4, 0xFF] {
        let mut bytes = HK_RESPONSE_REF;
        bytes[1] = frame_type;
        assert_eq!(
            BusResponse::decode(&bytes),
            Err(DecodeError::UnexpectedFrameType)
        );
    }
}

#[test]
fn bus_response_rejects_unknown_status() {
    // ICD: only 0 (OK) and 1 (NO_RESPONSE) exist.
    for status in [2, 0x10, 0xFF] {
        let mut bytes = HK_RESPONSE_REF;
        bytes[6] = status;
        assert_eq!(BusResponse::decode(&bytes), Err(DecodeError::InvalidStatus));
    }
}

#[test]
fn a_bus_request_is_not_a_bus_response() {
    // Same layout: only the frame type tells them apart.
    assert_eq!(
        BusResponse::decode(&GET_HK_REQUEST_REF),
        Err(DecodeError::UnexpectedFrameType)
    );
    assert_eq!(
        BusRequest::decode(&HK_RESPONSE_REF),
        Err(DecodeError::UnexpectedFrameType)
    );
}

// --- Both ---

#[test]
fn a_tick_is_not_a_cycle_done() {
    // Same size and same layout: only the frame type tells them apart.
    assert_eq!(
        CycleDone::decode(&TICK_REF),
        Err(DecodeError::UnexpectedFrameType)
    );
    assert_eq!(
        Tick::decode(&CYCLE_DONE_REF),
        Err(DecodeError::UnexpectedFrameType)
    );
}

#[test]
fn all_zero_buffer_is_not_a_frame() {
    assert!(Tick::decode(&[0; Tick::LEN]).is_err());
    assert!(CycleDone::decode(&[0; CycleDone::LEN]).is_err());
}
