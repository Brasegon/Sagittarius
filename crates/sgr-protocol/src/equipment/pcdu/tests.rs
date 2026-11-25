//! Unit tests of the PCDU formats, checked against the reference frames of `docs/icd.md`.

use super::*;

// Housekeeping payloads, copied from the `BusResponse` reference frames of docs/icd.md
// (the bytes that follow the 6-byte header and the status).

/// `BAT_V` 28 000 mV, `BAT_TEMP` -5.25 °C, every switch open
const HK_REF_A: [u8; 5] = [0x6D, 0x60, 0xFD, 0xF3, 0x00];
/// `BAT_V` 33 600 mV, `BAT_TEMP` 21.50 °C, `BAT_HEATER` closed
const HK_REF_B: [u8; 5] = [0x83, 0x40, 0x08, 0x66, 0x01];

const HK_A: PcduHk = PcduHk {
    bat_v: 28_000,
    bat_temp: -525,
    switches: 0,
};
const HK_B: PcduHk = PcduHk {
    bat_v: 33_600,
    bat_temp: 2_150,
    switches: SWITCH_BAT_HEATER,
};

#[test]
fn constants_match_the_icd() {
    assert_eq!(PCDU_ADDRESS, 0x10);
    assert_eq!(SWITCH_BAT_HEATER, 0b0000_0001);
    assert_eq!(PcduHk::LEN, 5);
}

// --- Housekeeping (response to GET_HK) ---

#[test]
fn hk_encodes_reference_payloads() {
    assert_eq!(HK_A.encode(), HK_REF_A);
    assert_eq!(HK_B.encode(), HK_REF_B);
}

#[test]
fn hk_decodes_reference_payloads() {
    assert_eq!(PcduHk::decode(&HK_REF_A), Some(HK_A));
    assert_eq!(PcduHk::decode(&HK_REF_B), Some(HK_B));
}

#[test]
fn hk_round_trips_extreme_values() {
    for hk in [
        PcduHk {
            bat_v: 0,
            bat_temp: 0,
            switches: 0,
        },
        PcduHk {
            bat_v: u16::MAX,
            bat_temp: i16::MIN,
            switches: 0xFF,
        },
        PcduHk {
            bat_v: u16::MAX,
            bat_temp: i16::MAX,
            switches: 0xFF,
        },
    ] {
        assert_eq!(PcduHk::decode(&hk.encode()), Some(hk));
    }
}

#[test]
fn hk_rejects_wrong_length() {
    // Empty, one byte short, one byte long.
    assert_eq!(PcduHk::decode(&[]), None);
    assert_eq!(PcduHk::decode(&HK_REF_A[..4]), None);
    let mut too_long = [0u8; 6];
    too_long[..5].copy_from_slice(&HK_REF_A);
    assert_eq!(PcduHk::decode(&too_long), None);
}

#[test]
fn hk_tells_whether_the_heater_is_on() {
    // How the flight software knows the state of the heater: bit 0 of `switches`.
    let open = PcduHk::decode(&HK_REF_A).unwrap();
    assert_eq!(open.switches & SWITCH_BAT_HEATER, 0);

    let closed = PcduHk::decode(&HK_REF_B).unwrap();
    assert_ne!(closed.switches & SWITCH_BAT_HEATER, 0);
}

// --- Requests (GET_HK and SET_SWITCHES) ---

// Request payloads, copied from the `BusRequest` reference frames of docs/icd.md
// (the bytes that follow the 6-byte header and the address).

/// `GET_HK`
const GET_HK_REF: [u8; 1] = [0x01];
/// `SET_SWITCHES`, `BAT_HEATER` closed
const SET_SWITCHES_REF: [u8; 2] = [0x02, 0x01];

/// The request that closes the heater switch.
const HEATER_ON: PcduRequest = PcduRequest::SetSwitches {
    switches: SWITCH_BAT_HEATER,
};

#[test]
fn request_encodes_reference_payloads() {
    assert_eq!(&PcduRequest::GetHk.encode()[..], &GET_HK_REF[..]);
    assert_eq!(&HEATER_ON.encode()[..], &SET_SWITCHES_REF[..]);
}

#[test]
fn request_decodes_reference_payloads() {
    assert_eq!(PcduRequest::decode(&GET_HK_REF), Some(PcduRequest::GetHk));
    assert_eq!(PcduRequest::decode(&SET_SWITCHES_REF), Some(HEATER_ON));
}

#[test]
fn request_travels_in_a_bus_frame() {
    // `encode` returns a `Payload`, ready to be carried by a `BusRequest`. The result is the
    // fourth reference frame of the ICD: tick 1, to the PCDU, SET_SWITCHES, BAT_HEATER closed.
    let frame = crate::lockstep::BusRequest {
        tick: 1,
        address: PCDU_ADDRESS,
        payload: HEATER_ON.encode(),
    };
    assert_eq!(
        &frame.encode()[..],
        &[0x02, 0x02, 0x00, 0x00, 0x00, 0x01, 0x10, 0x02, 0x01][..]
    );
}

#[test]
fn request_carries_every_switches_value() {
    // SET_SWITCHES is a state command: one byte gives every switch at once. The codec carries
    // it untouched: the ICD says that reserved bits are ignored on reception, not refused.
    for switches in 0..=u8::MAX {
        let request = PcduRequest::SetSwitches { switches };
        let bytes = request.encode();
        assert_eq!(&bytes[..], &[0x02, switches][..]);
        assert_eq!(PcduRequest::decode(&bytes), Some(request));
    }
}

#[test]
fn request_rejects_unknown_code() {
    // 0x01 and 0x02 are the only commands of the ICD. Any other code is refused, whether it
    // comes alone or with an argument.
    for code in 0..=u8::MAX {
        if code == 0x01 || code == 0x02 {
            continue;
        }
        assert_eq!(PcduRequest::decode(&[code]), None);
        assert_eq!(PcduRequest::decode(&[code, 0x01]), None);
    }
}

#[test]
fn request_rejects_wrong_length() {
    assert_eq!(PcduRequest::decode(&[]), None);
    // GET_HK takes no argument.
    assert_eq!(PcduRequest::decode(&[0x01, 0x00]), None);
    // SET_SWITCHES takes exactly one.
    assert_eq!(PcduRequest::decode(&[0x02]), None);
    assert_eq!(PcduRequest::decode(&[0x02, 0x01, 0x00]), None);
}
