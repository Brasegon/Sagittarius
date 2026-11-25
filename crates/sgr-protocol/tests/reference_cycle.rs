//! The reference cycle of `docs/icd.md`, played through the public API only.
//!
//! A file in `tests/` is compiled as a separate crate. Like `sgr-flight` and `sgr-sim`, it sees
//! only what is `pub`: a missing `pub` breaks the build here, where the unit tests, which live
//! inside the modules they test, would not notice it.

use sgr_protocol::equipment::pcdu::{PCDU_ADDRESS, PcduHk, PcduRequest, SWITCH_BAT_HEATER};
use sgr_protocol::lockstep::{BusRequest, BusResponse, BusStatus, CycleDone, Payload, Tick};

// The first six reference frames of docs/icd.md: one complete cycle, in order.

const TICK: [u8; 6] = [0x02, 0x01, 0x00, 0x00, 0x00, 0x01];
const GET_HK_REQUEST: [u8; 8] = [0x02, 0x02, 0x00, 0x00, 0x00, 0x01, 0x10, 0x01];
const HK_RESPONSE: [u8; 12] = [
    0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x6D, 0x60, 0xFD, 0xF3, 0x00,
];
const SET_SWITCHES_REQUEST: [u8; 9] = [0x02, 0x02, 0x00, 0x00, 0x00, 0x01, 0x10, 0x02, 0x01];
const EMPTY_RESPONSE: [u8; 7] = [0x02, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00];
const CYCLE_DONE: [u8; 6] = [0x02, 0x04, 0x00, 0x00, 0x00, 0x01];

/// What the PCDU measures during this cycle: 28 000 mV, -5.25 °C, every switch open.
const HK: PcduHk = PcduHk {
    bat_v: 28_000,
    bat_temp: -525,
    switches: 0,
};

/// The request that closes the heater switch.
const HEATER_ON: PcduRequest = PcduRequest::SetSwitches {
    switches: SWITCH_BAT_HEATER,
};

/// The cycle as the flight software lives it: it decodes what the simulator sends and encodes
/// what its PCDU driver asks.
#[test]
fn flight_software_side_of_the_cycle() {
    // The simulator gives the tick.
    let tick = Tick::decode(&TICK).unwrap().tick;
    assert_eq!(tick, 1);

    // The driver asks the PCDU for its housekeeping...
    let get_hk = BusRequest {
        tick,
        address: PCDU_ADDRESS,
        payload: PcduRequest::GetHk.encode(),
    };
    assert_eq!(&get_hk.encode()[..], &GET_HK_REQUEST[..]);

    // ...and reads the answer: the battery is cold and its heater is off.
    let response = BusResponse::decode(&HK_RESPONSE).unwrap();
    assert_eq!(response.status, BusStatus::Ok);
    let hk = PcduHk::decode(&response.payload).unwrap();
    assert_eq!(hk, HK);
    assert_eq!(hk.bat_v, 28_000);
    assert_eq!(hk.bat_temp, -525);
    assert_eq!(hk.switches & SWITCH_BAT_HEATER, 0);

    // So it closes the heater switch, and the PCDU acknowledges with an empty response.
    let heater_on = BusRequest {
        tick,
        address: PCDU_ADDRESS,
        payload: HEATER_ON.encode(),
    };
    assert_eq!(&heater_on.encode()[..], &SET_SWITCHES_REQUEST[..]);
    let ack = BusResponse::decode(&EMPTY_RESPONSE).unwrap();
    assert_eq!(ack.status, BusStatus::Ok);
    assert!(ack.payload.is_empty());

    // The flight software has finished its cycle.
    assert_eq!(CycleDone { tick }.encode(), CYCLE_DONE);
}

/// The same cycle as the simulator lives it: it decodes what the flight software sends and
/// encodes what its PCDU model answers.
#[test]
fn simulator_side_of_the_cycle() {
    assert_eq!(Tick { tick: 1 }.encode(), TICK);

    // The PCDU model receives GET_HK and answers with its housekeeping.
    let request = BusRequest::decode(&GET_HK_REQUEST).unwrap();
    assert_eq!(request.address, PCDU_ADDRESS);
    assert_eq!(
        PcduRequest::decode(&request.payload),
        Some(PcduRequest::GetHk)
    );
    let hk_bytes: [u8; PcduHk::LEN] = HK.encode();
    let response = BusResponse {
        tick: request.tick,
        status: BusStatus::Ok,
        payload: Payload::from_array(hk_bytes),
    };
    assert_eq!(&response.encode()[..], &HK_RESPONSE[..]);

    // It receives SET_SWITCHES and acknowledges with an empty response.
    let request = BusRequest::decode(&SET_SWITCHES_REQUEST).unwrap();
    assert_eq!(request.address, PCDU_ADDRESS);
    assert_eq!(PcduRequest::decode(&request.payload), Some(HEATER_ON));
    let ack = BusResponse {
        tick: request.tick,
        status: BusStatus::Ok,
        payload: Payload::new(),
    };
    assert_eq!(&ack.encode()[..], &EMPTY_RESPONSE[..]);

    // The flight software says it has finished: the world can move one step forward.
    assert_eq!(CycleDone::decode(&CYCLE_DONE), Ok(CycleDone { tick: 1 }));
}
