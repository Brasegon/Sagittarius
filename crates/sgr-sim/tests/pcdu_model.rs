//! The model of the PCDU: what the simulator answers when the flight software questions it.
//!
//! The model keeps the real state of the switches and measures the world. It speaks the format
//! of the ICD (`docs/icd.md`, section "PCDU"), shared with the driver of the flight software.
//!
//! Rules of the model: `docs/decisions/0005-simulateur-minimal.md`.

use sgr_protocol::equipment::pcdu::{PcduHk, PcduRequest, SWITCH_BAT_HEATER};
use sgr_protocol::lockstep::Payload;
use sgr_sim::equipment::pcdu::Pcdu;
use sgr_sim::world::World;

/// A world whose battery is at `bat_v` volts and `bat_temp` degrees Celsius.
fn world(bat_v: f64, bat_temp: f64) -> World {
    World { bat_temp, bat_v }
}

/// Sends `GET_HK` to the model and decodes its answer.
fn get_hk(pcdu: &mut Pcdu, world: &World) -> PcduHk {
    let response = pcdu
        .handle(world, &PcduRequest::GetHk.encode())
        .expect("the PCDU answers GET_HK");

    PcduHk::decode(&response).expect("the answer to GET_HK is a housekeeping")
}

/// Sends `SET_SWITCHES` to the model and returns its answer.
fn set_switches(pcdu: &mut Pcdu, world: &World, switches: u8) -> Option<Payload> {
    pcdu.handle(world, &PcduRequest::SetSwitches { switches }.encode())
}

// --- GET_HK ---

#[test]
fn new_pcdu_has_every_switch_open() {
    let mut pcdu = Pcdu::new();

    assert!(!pcdu.heater_on());
    assert_eq!(get_hk(&mut pcdu, &world(28.0, 20.0)).switches, 0);
}

#[test]
fn get_hk_gives_the_reference_frame_of_the_icd() {
    let mut pcdu = Pcdu::new();

    let response = pcdu.handle(&world(28.0, -5.25), &[0x01]);

    // ICD, "Trames de référence": 28 000 mV, -5.25 °C, every switch open. The bytes are written
    // here by hand: an error shared by the model and the driver would not hide.
    let expected = Payload::from_array([0x6D, 0x60, 0xFD, 0xF3, 0x00]);
    assert_eq!(response, Some(expected));
}

#[test]
fn get_hk_measures_the_world_at_the_time_of_the_request() {
    let mut pcdu = Pcdu::new();

    let cold = get_hk(&mut pcdu, &world(28.0, -5.25));
    let warm = get_hk(&mut pcdu, &world(33.6, 21.5));

    assert_eq!(cold.bat_temp, -525);
    assert_eq!(cold.bat_v, 28_000);
    assert_eq!(warm.bat_temp, 2_150);
    assert_eq!(warm.bat_v, 33_600);
}

// --- SET_SWITCHES ---

#[test]
fn set_switches_closes_the_heater_at_once() {
    let mut pcdu = Pcdu::new();
    let world = world(33.6, 21.5);

    let response = set_switches(&mut pcdu, &world, SWITCH_BAT_HEATER);

    // An empty answer, not a silence: the PCDU obeyed and has nothing to say.
    assert_eq!(response, Some(Payload::new()));
    assert!(pcdu.heater_on());

    // A GET_HK sent in the same cycle already shows the new state (ICD, "switches"). This is the
    // second housekeeping of the reference frames.
    let hk = pcdu.handle(&world, &[0x01]);
    assert_eq!(
        hk,
        Some(Payload::from_array([0x83, 0x40, 0x08, 0x66, 0x01]))
    );
}

#[test]
fn set_switches_is_a_state_command() {
    let mut pcdu = Pcdu::new();
    let world = world(28.0, 20.0);

    // The command gives the state of every switch: sending it twice changes nothing more, and
    // a 0 opens what a 1 had closed.
    set_switches(&mut pcdu, &world, SWITCH_BAT_HEATER);
    set_switches(&mut pcdu, &world, SWITCH_BAT_HEATER);
    assert!(pcdu.heater_on());

    set_switches(&mut pcdu, &world, 0);
    assert!(!pcdu.heater_on());
    assert_eq!(get_hk(&mut pcdu, &world).switches, 0);
}

#[test]
fn reserved_bits_are_ignored() {
    let mut pcdu = Pcdu::new();
    let world = world(28.0, 20.0);

    // ICD, "Conventions": reserved bits are ignored on reception. The housekeeping gives the
    // real state of the switches, and only the battery heater exists.
    set_switches(&mut pcdu, &world, 0xFF);
    assert!(pcdu.heater_on());
    assert_eq!(get_hk(&mut pcdu, &world).switches, SWITCH_BAT_HEATER);

    set_switches(&mut pcdu, &world, 0xFE);
    assert!(!pcdu.heater_on());
    assert_eq!(get_hk(&mut pcdu, &world).switches, 0);
}

// --- A request the PCDU does not understand ---

#[test]
fn request_that_is_not_understood_gets_no_answer() {
    let mut pcdu = Pcdu::new();
    let world = world(28.0, 20.0);

    // ICD: "Une demande dont le code est inconnu ou dont la longueur est fausse reste sans
    // réponse". `None` is that silence.
    assert_eq!(pcdu.handle(&world, &[]), None);
    assert_eq!(pcdu.handle(&world, &[0x7F]), None);
    assert_eq!(pcdu.handle(&world, &[0x01, 0x00]), None);
    assert_eq!(pcdu.handle(&world, &[0x02]), None);
    assert_eq!(pcdu.handle(&world, &[0x02, 0x01, 0x01]), None);
}

#[test]
fn request_that_is_not_understood_changes_nothing() {
    let mut pcdu = Pcdu::new();
    let world = world(28.0, 20.0);
    set_switches(&mut pcdu, &world, SWITCH_BAT_HEATER);

    pcdu.handle(&world, &[0x02]);
    pcdu.handle(&world, &[0x02, 0x00, 0x00]);
    pcdu.handle(&world, &[0x7F, 0x00]);

    assert!(pcdu.heater_on());
}

// --- From physical units to the wire ---

#[test]
fn measures_are_rounded_to_the_nearest_step() {
    let mut pcdu = Pcdu::new();

    // 0.29 * 100.0 gives 28.999999999999996 in floating point: cutting the decimals off would
    // give 28. Rounding gives 29.
    assert_eq!(get_hk(&mut pcdu, &world(28.0, 0.29)).bat_temp, 29);
    assert_eq!(get_hk(&mut pcdu, &world(28.0, -0.29)).bat_temp, -29);

    assert_eq!(get_hk(&mut pcdu, &world(28.0, 21.504)).bat_temp, 2_150);
    assert_eq!(get_hk(&mut pcdu, &world(28.0, 21.506)).bat_temp, 2_151);
    assert_eq!(get_hk(&mut pcdu, &world(28.0, -5.254)).bat_temp, -525);
    assert_eq!(get_hk(&mut pcdu, &world(28.0, -5.256)).bat_temp, -526);

    assert_eq!(get_hk(&mut pcdu, &world(28.0004, 20.0)).bat_v, 28_000);
    assert_eq!(get_hk(&mut pcdu, &world(28.0006, 20.0)).bat_v, 28_001);
}

#[test]
fn measures_saturate_at_the_bounds_of_their_type() {
    let mut pcdu = Pcdu::new();

    // ICD, "Capteurs": BAT_TEMP goes from -327.68 to +327.67 °C, BAT_V from 0 to 65.535 V. A
    // value outside the range is sent as the nearest bound, never as a value that wrapped
    // around.
    assert_eq!(get_hk(&mut pcdu, &world(28.0, 400.0)).bat_temp, i16::MAX);
    assert_eq!(get_hk(&mut pcdu, &world(28.0, -400.0)).bat_temp, i16::MIN);
    assert_eq!(get_hk(&mut pcdu, &world(70.0, 20.0)).bat_v, u16::MAX);
    assert_eq!(get_hk(&mut pcdu, &world(-1.0, 20.0)).bat_v, 0);
}
