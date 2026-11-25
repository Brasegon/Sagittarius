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
