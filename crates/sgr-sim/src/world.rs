//! The physical world around the equipment, in physical units: degrees Celsius, volts, watts.
//!
//! For now, a single thermal node: the battery. It loses heat to its environment and gains some
//! when its heater is powered. The world knows nothing of the equipment bus: the equipment
//! models measure it, and the simulator moves it forward one tick at a time.
//!
//! Model and values: `docs/decisions/0005-simulateur-minimal.md`.

/// Duration of one tick, in seconds: the world moves forward by this much at every step.
pub const TICK_SECONDS: f64 = 0.1;

/// Temperature of what surrounds the battery, in degrees Celsius. This is the cold case: the
/// battery heads for it when its heater is off.
pub const ENV_TEMP: f64 = -10.0;

/// Heat capacity of the battery, in joules per degree: the energy it takes to warm it by 1 °C.
pub const BAT_HEAT_CAPACITY: f64 = 400.0;

/// Thermal resistance between the battery and its environment, in degrees per watt: with a gap
/// of 5 °C, the battery loses 1 W.
pub const BAT_THERMAL_RESISTANCE: f64 = 5.0;

/// Power of the battery heater when it is powered, in watts.
pub const BAT_HEATER_POWER: f64 = 6.0;

/// State of the physical world at one instant.
///
/// The fields are public: a scenario sets the starting conditions, and a test reads the truth
/// to compare it with what the equipment measures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
    /// Temperature of the battery, in degrees Celsius.
    pub bat_temp: f64,
    /// Voltage of the battery, in volts. It does not vary yet.
    pub bat_v: f64,
}

impl World {
    /// Moves the world forward by one tick of [`TICK_SECONDS`].
    ///
    /// The battery loses a power proportional to its gap to [`ENV_TEMP`], and receives
    /// [`BAT_HEATER_POWER`] if `heater_on`. The net power, applied for one tick, changes its
    /// temperature through [`BAT_HEAT_CAPACITY`].
    pub fn step(&mut self, heater_on: bool) {
        let power_lost = (self.bat_temp - ENV_TEMP) / BAT_THERMAL_RESISTANCE;
        let power_in = if heater_on { BAT_HEATER_POWER } else { 0.0 };

        self.bat_temp += (power_in - power_lost) * TICK_SECONDS / BAT_HEAT_CAPACITY;
    }
}
