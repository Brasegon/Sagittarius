//! The physical world of the simulator: for now, the temperature of the battery.
//!
//! One thermal node: the battery loses heat to its environment and gains some when its heater
//! is powered. Model and values: `docs/decisions/0005-simulateur-minimal.md`.
//!
//! A file in `tests/` is compiled as a separate crate: it sees only what is `pub`.

use sgr_sim::world::{
    BAT_HEAT_CAPACITY, BAT_HEATER_POWER, BAT_THERMAL_RESISTANCE, ENV_TEMP, TICK_SECONDS, World,
};

/// Temperature at which the battery settles when its heater stays on: +20 °C.
const EQUILIBRIUM: f64 = ENV_TEMP + BAT_HEATER_POWER * BAT_THERMAL_RESISTANCE;

/// Time constant of the battery: 2000 s. After that long, its gap to the temperature it is
/// heading for has shrunk to 37 % of what it was.
const TIME_CONSTANT: f64 = BAT_THERMAL_RESISTANCE * BAT_HEAT_CAPACITY;

/// Fails unless `actual` is within `tolerance` of `expected`.
///
/// Floats are not compared with `==`: two ways of computing the same value rarely agree on the
/// last digits.
fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected} ± {tolerance}, got {actual}"
    );
}

/// A world whose battery is at `bat_temp` degrees Celsius.
fn world_at(bat_temp: f64) -> World {
    World {
        bat_temp,
        bat_v: 28.0,
    }
}

/// Moves the world forward by `seconds`, the heater staying in the same state.
fn run(world: &mut World, seconds: f64, heater_on: bool) {
    let steps = (seconds / TICK_SECONDS).round() as u32;
    for _ in 0..steps {
        world.step(heater_on);
    }
}

// --- One step ---

#[test]
fn one_step_without_heater_cools_the_battery() {
    let mut world = world_at(20.0);

    world.step(false);

    // The battery is 30 °C above its environment: it loses 30 / 5 = 6 W.
    // In 0.1 s, that is 0.6 J. With 400 J per degree, it cools by 0.0015 °C.
    assert_close(world.bat_temp, 19.9985, 1e-9);
}

#[test]
fn one_step_with_heater_warms_the_battery() {
    let mut world = world_at(-10.0);

    world.step(true);

    // The battery is at the temperature of its environment: it loses nothing.
    // The heater brings 6 W: 0.6 J in 0.1 s, which is 0.0015 °C.
    assert_close(world.bat_temp, -9.9985, 1e-9);
}

#[test]
fn step_adds_the_heater_and_the_loss() {
    let mut world = world_at(0.0);

    world.step(true);

    // Loss: 10 / 5 = 2 W. Heater: 6 W. Net: 4 W, so 0.4 J in 0.1 s, which is 0.001 °C.
    assert_close(world.bat_temp, 0.001, 1e-9);
}

#[test]
fn step_does_not_touch_the_voltage() {
    let mut world = world_at(20.0);

    world.step(true);
    world.step(false);

    assert_close(world.bat_v, 28.0, 1e-12);
}

// --- Without heater ---

#[test]
fn battery_at_the_temperature_of_its_environment_stays_there() {
    let mut world = world_at(ENV_TEMP);

    run(&mut world, 100.0, false);

    assert_close(world.bat_temp, ENV_TEMP, 1e-9);
}

#[test]
fn battery_cools_at_every_step_while_it_is_warmer_than_its_environment() {
    let mut world = world_at(20.0);

    for _ in 0..1_000 {
        let before = world.bat_temp;
        world.step(false);
        assert!(world.bat_temp < before);
    }
}

#[test]
fn battery_cools_down_to_its_environment_and_no_further() {
    let mut world = world_at(20.0);

    run(&mut world, 10.0 * TIME_CONSTANT, false);

    assert_close(world.bat_temp, ENV_TEMP, 0.01);
    assert!(world.bat_temp >= ENV_TEMP);
}

#[test]
fn battery_colder_than_its_environment_warms_up_to_it() {
    let mut world = world_at(-40.0);

    run(&mut world, 10.0 * TIME_CONSTANT, false);

    assert_close(world.bat_temp, ENV_TEMP, 0.01);
    assert!(world.bat_temp <= ENV_TEMP);
}

#[test]
fn gap_to_the_environment_shrinks_to_37_percent_in_one_time_constant() {
    let mut world = world_at(20.0);

    run(&mut world, TIME_CONSTANT, false);

    // The law of the model is an exponential: after one time constant, the gap of 30 °C is
    // multiplied by 1 / e. A wrong time step or a wrong heat capacity shows up here.
    let expected = ENV_TEMP + 30.0 / std::f64::consts::E;
    assert_close(world.bat_temp, expected, 0.01);
}

// --- With heater ---

#[test]
fn heater_holds_the_battery_at_its_equilibrium() {
    let mut world = world_at(EQUILIBRIUM);

    run(&mut world, 100.0, true);

    // At +20 °C, the battery loses 6 W, exactly what the heater brings.
    assert_close(world.bat_temp, 20.0, 1e-9);
}

#[test]
fn heater_warms_the_battery_up_to_its_equilibrium_and_no_further() {
    let mut world = world_at(ENV_TEMP);

    run(&mut world, 10.0 * TIME_CONSTANT, true);

    assert_close(world.bat_temp, EQUILIBRIUM, 0.01);
    assert!(world.bat_temp <= EQUILIBRIUM);
}

#[test]
fn heater_is_strong_enough_for_the_thermostat() {
    // The thermostat of the flight software turns the heater on below +5 °C and off above
    // +10 °C: from the first threshold, the heater must bring the battery past the second.
    let mut world = world_at(5.0);

    run(&mut world, 3_600.0, true);

    assert!(world.bat_temp > 10.0);
}
