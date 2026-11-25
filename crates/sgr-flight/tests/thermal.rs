//! The thermostat of the battery heater, used from outside the crate.
//!
//! Temperatures are in hundredths of a degree Celsius, like `bat_temp`: 500 is +5.00 °C.
//! The heater turns on below +5.00 °C, turns off above +10.00 °C, and keeps its state in between.

use sgr_flight::thermal::Thermostat;

#[test]
fn heater_is_off_at_start() {
    let thermostat = Thermostat::new();

    assert!(!thermostat.heater_on());
}

#[test]
fn heater_turns_on_below_five_degrees() {
    let mut thermostat = Thermostat::new();

    thermostat.update(200); // +2.00 °C
    assert!(thermostat.heater_on());
}

#[test]
fn heater_turns_on_at_minus_five_degrees() {
    let mut thermostat = Thermostat::new();

    thermostat.update(-500); // -5.00 °C
    assert!(thermostat.heater_on());
}

#[test]
fn heater_turns_off_above_ten_degrees() {
    let mut thermostat = Thermostat::new();

    // Cold first, so that the heater is on: otherwise nothing proves that it turns off.
    thermostat.update(200); // +2.00 °C
    assert!(thermostat.heater_on());

    thermostat.update(1_200); // +12.00 °C
    assert!(!thermostat.heater_on());
}

#[test]
fn heater_stays_on_between_five_and_ten_degrees() {
    let mut thermostat = Thermostat::new();

    thermostat.update(200); // +2.00 °C
    assert!(thermostat.heater_on());

    thermostat.update(700); // +7.00 °C
    assert!(thermostat.heater_on());
}

#[test]
fn heater_stays_off_between_five_and_ten_degrees() {
    let mut thermostat = Thermostat::new();

    // Never been cold: still off.
    thermostat.update(700); // +7.00 °C
    assert!(!thermostat.heater_on());

    // Back from a warm battery: still off.
    thermostat.update(1_500); // +15.00 °C
    thermostat.update(700); // +7.00 °C
    assert!(!thermostat.heater_on());
}

#[test]
fn heater_keeps_its_state_when_the_same_temperature_comes_again() {
    let mut thermostat = Thermostat::new();

    // The thermostat is called at every cycle: most of the time, nothing has changed.
    thermostat.update(200); // +2.00 °C
    thermostat.update(200);
    assert!(thermostat.heater_on());

    thermostat.update(1_200); // +12.00 °C
    thermostat.update(1_200);
    assert!(!thermostat.heater_on());
}

#[test]
fn heater_turns_on_below_five_degrees_but_not_at_five() {
    let mut thermostat = Thermostat::new();

    thermostat.update(500); // +5.00 °C exactly: nothing changes
    assert!(!thermostat.heater_on());

    thermostat.update(499); // +4.99 °C
    assert!(thermostat.heater_on());
}

#[test]
fn heater_turns_off_above_ten_degrees_but_not_at_ten() {
    let mut thermostat = Thermostat::new();
    thermostat.update(499); // +4.99 °C
    assert!(thermostat.heater_on());

    thermostat.update(1_000); // +10.00 °C exactly: nothing changes
    assert!(thermostat.heater_on());

    thermostat.update(1_001); // +10.01 °C
    assert!(!thermostat.heater_on());
}

#[test]
fn heater_follows_a_battery_that_cools_warms_and_cools_again() {
    let mut thermostat = Thermostat::new();

    // What the heater must do for each temperature, in order.
    for (bat_temp, expected) in [
        (800, false),   // +8.00 °C: off since the start
        (300, true),    // +3.00 °C: turns on
        (800, true),    // +8.00 °C: stays on while the battery warms up
        (1_100, false), // +11.00 °C: turns off
        (800, false),   // +8.00 °C: stays off while the battery cools down
        (300, true),    // +3.00 °C: turns on again
    ] {
        thermostat.update(bat_temp);
        assert_eq!(thermostat.heater_on(), expected, "at {bat_temp}");
    }
}

#[test]
fn heater_handles_the_extreme_temperatures() {
    let mut thermostat = Thermostat::new();

    thermostat.update(i16::MIN); // -327.68 °C
    assert!(thermostat.heater_on());

    thermostat.update(i16::MAX); // +327.67 °C
    assert!(!thermostat.heater_on());
}
