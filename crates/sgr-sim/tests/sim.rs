//! The simulator as the flight software sees it during a lockstep cycle: a tick, a bus, and a
//! world that moves forward one step when the cycle is over.
//!
//! Cycle: `docs/icd.md`, "Déroulement d'un cycle". Shape of the simulator:
//! `docs/decisions/0005-simulateur-minimal.md`.

use sgr_protocol::equipment::pcdu::{PCDU_ADDRESS, PcduHk, PcduRequest, SWITCH_BAT_HEATER};
use sgr_protocol::lockstep::Payload;
use sgr_sim::sim::Sim;
use sgr_sim::world::World;

/// Fails unless `actual` is within `tolerance` of `expected`.
fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected} ± {tolerance}, got {actual}"
    );
}

/// A simulator whose battery is at 28 V and `bat_temp` degrees Celsius.
fn sim_at(bat_temp: f64) -> Sim {
    Sim::new(World {
        bat_temp,
        bat_v: 28.0,
    })
}

/// Asks the PCDU for its housekeeping through the bus, and decodes it.
fn get_hk(sim: &mut Sim) -> PcduHk {
    let response = sim
        .bus_request(PCDU_ADDRESS, &PcduRequest::GetHk.encode())
        .expect("the PCDU answers GET_HK");

    PcduHk::decode(&response).expect("the answer to GET_HK is a housekeeping")
}

/// Commands the switches of the PCDU through the bus.
fn set_switches(sim: &mut Sim, switches: u8) -> Option<Payload> {
    sim.bus_request(
        PCDU_ADDRESS,
        &PcduRequest::SetSwitches { switches }.encode(),
    )
}

// --- Start and tick ---

#[test]
fn new_sim_starts_at_tick_0_with_the_given_world() {
    let world = World {
        bat_temp: -5.25,
        bat_v: 33.6,
    };

    let sim = Sim::new(world);

    assert_eq!(sim.tick(), 0);
    assert_eq!(sim.world, world);
    assert!(!sim.pcdu.heater_on());
}

#[test]
fn step_moves_to_the_next_tick() {
    let mut sim = sim_at(20.0);

    sim.step();
    assert_eq!(sim.tick(), 1);

    sim.step();
    sim.step();
    assert_eq!(sim.tick(), 3);
}

#[test]
fn bus_request_does_not_move_the_tick() {
    let mut sim = sim_at(20.0);

    get_hk(&mut sim);
    set_switches(&mut sim, SWITCH_BAT_HEATER);

    assert_eq!(sim.tick(), 0);
}

// --- The bus ---

#[test]
fn bus_request_reaches_the_pcdu_at_its_address() {
    let mut sim = sim_at(-5.25);

    let response = sim.bus_request(0x10, &[0x01]);

    // ICD, "Trames de référence": the payload of the first `BusResponse`.
    let expected = Payload::from_array([0x6D, 0x60, 0xFD, 0xF3, 0x00]);
    assert_eq!(response, Some(expected));
}

#[test]
fn nobody_answers_at_an_unknown_address() {
    let mut sim = sim_at(20.0);
    let get_hk = PcduRequest::GetHk.encode();

    // ICD, "Absence de réponse": no equipment has this address.
    assert_eq!(sim.bus_request(0x00, &get_hk), None);
    assert_eq!(sim.bus_request(0x11, &get_hk), None);
    assert_eq!(sim.bus_request(0xFF, &get_hk), None);
}

#[test]
fn request_at_an_unknown_address_does_not_reach_the_pcdu() {
    let mut sim = sim_at(20.0);
    let close_heater = PcduRequest::SetSwitches {
        switches: SWITCH_BAT_HEATER,
    }
    .encode();

    sim.bus_request(0x11, &close_heater);

    assert!(!sim.pcdu.heater_on());
}

#[test]
fn silence_of_the_pcdu_goes_through_the_bus() {
    let mut sim = sim_at(20.0);

    // The address is right, but the PCDU does not understand the request.
    assert_eq!(sim.bus_request(PCDU_ADDRESS, &[0x7F]), None);
    assert_eq!(sim.bus_request(PCDU_ADDRESS, &[]), None);
}

// --- The world is frozen during the cycle ---

#[test]
fn world_does_not_move_during_the_cycle() {
    let mut sim = sim_at(-5.25);
    let before = sim.world;

    let first = get_hk(&mut sim);
    set_switches(&mut sim, SWITCH_BAT_HEATER);
    let second = get_hk(&mut sim);

    // ICD: every measure read during cycle N is that of the same instant. Only `step` moves the
    // world.
    assert_eq!(sim.world, before);
    assert_eq!(first.bat_temp, second.bat_temp);
}

#[test]
fn command_changes_the_equipment_at_once() {
    let mut sim = sim_at(-5.25);

    let response = set_switches(&mut sim, SWITCH_BAT_HEATER);

    assert_eq!(response, Some(Payload::new()));
    assert!(sim.pcdu.heater_on());
    assert_eq!(get_hk(&mut sim).switches, SWITCH_BAT_HEATER);
}

// --- The step ---

#[test]
fn step_cools_the_battery_when_the_heater_is_off() {
    let mut sim = sim_at(20.0);

    sim.step();

    // Same value as in `tests/world.rs`: the step of the simulator is the step of the world.
    assert_close(sim.world.bat_temp, 19.9985, 1e-9);
}

#[test]
fn step_warms_the_battery_when_the_pcdu_powers_the_heater() {
    let mut sim = sim_at(-10.0);
    set_switches(&mut sim, SWITCH_BAT_HEATER);

    sim.step();

    // The effect of a command on the world comes with the step that follows it.
    assert_close(sim.world.bat_temp, -9.9985, 1e-9);
}

#[test]
fn step_follows_the_heater_from_one_cycle_to_the_next() {
    let mut sim = sim_at(-10.0);

    set_switches(&mut sim, SWITCH_BAT_HEATER);
    sim.step();
    let after_heating = sim.world.bat_temp;
    assert!(after_heating > -10.0);

    set_switches(&mut sim, 0);
    sim.step();
    assert!(sim.world.bat_temp < after_heating);
}

// --- A whole cycle ---

#[test]
fn reference_cycle_of_the_icd() {
    // The six reference frames of the ICD form one cycle. Here are their payloads, without the
    // lockstep frames around them.
    let mut sim = sim_at(-5.25);

    // `BusRequest` GET_HK, then its `BusResponse`.
    let hk = sim.bus_request(0x10, &[0x01]);
    assert_eq!(
        hk,
        Some(Payload::from_array([0x6D, 0x60, 0xFD, 0xF3, 0x00]))
    );

    // `BusRequest` SET_SWITCHES with the battery heater closed, then its empty `BusResponse`.
    let ack = sim.bus_request(0x10, &[0x02, 0x01]);
    assert_eq!(ack, Some(Payload::new()));

    // `CycleDone`: the world moves forward, and the next cycle starts.
    sim.step();
    assert_eq!(sim.tick(), 1);
    assert!(sim.world.bat_temp > -5.25);
    assert_eq!(get_hk(&mut sim).switches, SWITCH_BAT_HEATER);
}
