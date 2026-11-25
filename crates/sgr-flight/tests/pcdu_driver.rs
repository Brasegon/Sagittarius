//! The PCDU driver, used from outside the crate against fake buses.
//!
//! A file in `tests/` is compiled as a separate crate: it sees only what is `pub`, and it can
//! use `std` (`Vec`), which the driver itself cannot.

use sgr_flight::drivers::DriverError;
use sgr_flight::drivers::pcdu::{read_hk, set_switches};
use sgr_flight_core::platform::{BusError, EquipmentBus};
use sgr_protocol::equipment::pcdu::{PCDU_ADDRESS, PcduHk, PcduRequest, SWITCH_BAT_HEATER};

// Housekeeping payloads of the reference frames of docs/icd.md.

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

// --- A bus that follows a script ---

/// What the scripted bus does with every request.
enum Script {
    /// The equipment answers with these bytes.
    Answer(Vec<u8>),
    /// The equipment stays silent.
    Silence,
}

/// A bus that always reacts the same way, and remembers what it was asked.
struct ScriptedBus {
    script: Script,
    exchanges: u32,
    last_address: u8,
    last_request: Vec<u8>,
}

impl ScriptedBus {
    fn answering(bytes: &[u8]) -> Self {
        Self::new(Script::Answer(bytes.to_vec()))
    }

    fn silent() -> Self {
        Self::new(Script::Silence)
    }

    fn new(script: Script) -> Self {
        Self {
            script,
            exchanges: 0,
            last_address: 0,
            last_request: Vec::new(),
        }
    }
}

impl EquipmentBus for ScriptedBus {
    fn transfer(
        &mut self,
        address: u8,
        request: &[u8],
        response: &mut [u8],
    ) -> Result<usize, BusError> {
        self.exchanges += 1;
        self.last_address = address;
        self.last_request = request.to_vec();

        match &self.script {
            Script::Silence => Err(BusError::NoResponse),
            Script::Answer(bytes) => {
                if bytes.len() > response.len() {
                    return Err(BusError::ResponseTooLong);
                }
                response[..bytes.len()].copy_from_slice(bytes);
                Ok(bytes.len())
            }
        }
    }
}

// --- read_hk ---

#[test]
fn read_hk_sends_get_hk_to_the_pcdu() {
    let mut bus = ScriptedBus::answering(&HK_REF_A);

    assert_eq!(read_hk(&mut bus), Ok(HK_A));

    assert_eq!(bus.last_address, PCDU_ADDRESS);
    assert_eq!(bus.last_request, [0x01]);
    assert_eq!(bus.exchanges, 1);
}

#[test]
fn read_hk_decodes_the_answer_of_the_pcdu() {
    let hk = read_hk(&mut ScriptedBus::answering(&HK_REF_B));

    assert_eq!(hk, Ok(HK_B));
}

#[test]
fn read_hk_reports_a_silent_pcdu() {
    let mut bus = ScriptedBus::silent();

    assert_eq!(read_hk(&mut bus), Err(DriverError::NoResponse));

    // The driver does not retry by itself: trying again is a decision of the FDIR.
    assert_eq!(bus.exchanges, 1);
}

#[test]
fn read_hk_rejects_an_answer_of_the_wrong_length() {
    // Empty, one byte short, one byte long, as long as a bus frame can carry.
    let too_long = [HK_REF_A.as_slice(), &[0x00]].concat();
    for answer in [&[][..], &HK_REF_A[..4], &too_long, &[0u8; 32]] {
        let mut bus = ScriptedBus::answering(answer);

        assert_eq!(read_hk(&mut bus), Err(DriverError::InvalidResponse));
    }
}

// --- set_switches ---

#[test]
fn set_switches_sends_the_state_of_every_switch() {
    let mut bus = ScriptedBus::answering(&[]);

    assert_eq!(set_switches(&mut bus, SWITCH_BAT_HEATER), Ok(()));
    assert_eq!(bus.last_address, PCDU_ADDRESS);
    assert_eq!(bus.last_request, [0x02, 0x01]);

    assert_eq!(set_switches(&mut bus, 0), Ok(()));
    assert_eq!(bus.last_request, [0x02, 0x00]);

    assert_eq!(bus.exchanges, 2);
}

#[test]
fn set_switches_reports_a_silent_pcdu() {
    let mut bus = ScriptedBus::silent();

    assert_eq!(
        set_switches(&mut bus, SWITCH_BAT_HEATER),
        Err(DriverError::NoResponse)
    );
    assert_eq!(bus.exchanges, 1);
}

#[test]
fn set_switches_rejects_an_answer_that_is_not_empty() {
    // The ICD says that the PCDU answers SET_SWITCHES with an empty response.
    for answer in [&[0x00][..], &HK_REF_A] {
        let mut bus = ScriptedBus::answering(answer);

        assert_eq!(
            set_switches(&mut bus, SWITCH_BAT_HEATER),
            Err(DriverError::InvalidResponse)
        );
    }
}

// --- A bus that breaks its contract ---

/// A faulty bus: it claims a response of 200 bytes, whatever the size of the buffer.
struct LyingBus;

impl EquipmentBus for LyingBus {
    fn transfer(
        &mut self,
        _address: u8,
        _request: &[u8],
        _response: &mut [u8],
    ) -> Result<usize, BusError> {
        Ok(200)
    }
}

#[test]
fn drivers_survive_a_bus_that_lies_about_the_length() {
    // Flight code never panics. A length that comes from outside is checked before it is used:
    // `&buffer[..200]` on a small buffer would stop the whole flight software.
    assert_eq!(read_hk(&mut LyingBus), Err(DriverError::InvalidResponse));
    assert_eq!(
        set_switches(&mut LyingBus, SWITCH_BAT_HEATER),
        Err(DriverError::InvalidResponse)
    );
}

// --- The driver in front of a PCDU model ---

/// A bus with a small PCDU model behind it, built with the same `sgr-protocol` formats as the
/// driver. The model of `sgr-sim` will look like this.
struct PcduModel {
    hk: PcduHk,
}

impl EquipmentBus for PcduModel {
    fn transfer(
        &mut self,
        address: u8,
        request: &[u8],
        response: &mut [u8],
    ) -> Result<usize, BusError> {
        if address != PCDU_ADDRESS {
            return Err(BusError::NoResponse);
        }

        match PcduRequest::decode(request) {
            Some(PcduRequest::GetHk) => {
                let bytes = self.hk.encode();
                if bytes.len() > response.len() {
                    return Err(BusError::ResponseTooLong);
                }
                response[..bytes.len()].copy_from_slice(&bytes);
                Ok(bytes.len())
            }
            Some(PcduRequest::SetSwitches { switches }) => {
                self.hk.switches = switches;
                Ok(0)
            }
            None => Err(BusError::NoResponse),
        }
    }
}

#[test]
fn driver_and_pcdu_model_understand_each_other() {
    let mut pcdu = PcduModel { hk: HK_A };

    // The battery is cold and its heater is off.
    let hk = read_hk(&mut pcdu).unwrap();
    assert_eq!(hk.bat_temp, -525);
    assert_eq!(hk.switches & SWITCH_BAT_HEATER, 0);

    // The flight software closes the heater switch, then reads the housekeeping again to check
    // that the command was applied.
    assert_eq!(set_switches(&mut pcdu, SWITCH_BAT_HEATER), Ok(()));
    let hk = read_hk(&mut pcdu).unwrap();
    assert_ne!(hk.switches & SWITCH_BAT_HEATER, 0);
}

// --- DriverError ---

#[test]
fn driver_error_is_a_plain_value() {
    // Like `BusError`: it can be copied, compared and printed.
    let error = DriverError::NoResponse;
    let copy = error;
    assert_eq!(error, copy);
    assert_ne!(DriverError::NoResponse, DriverError::InvalidResponse);
    assert_eq!(format!("{error:?}"), "NoResponse");
}
