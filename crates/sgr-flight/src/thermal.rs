//! Thermal subsystem, minimal version: the thermostat of the battery heater.
//!
//! Temperatures are in hundredths of a degree Celsius, like `bat_temp` in the housekeeping of
//! the PCDU: 500 is +5.00 °C.

/// Thermostat with hysteresis for the battery heater.
///
/// The heater turns on below +5.00 °C and turns off above +10.00 °C. Between the two, it keeps
/// its state: with a single threshold, it would switch back and forth whenever the temperature
/// hovers around it. The battery is kept warm because a lithium-ion cell must not be charged
/// below 0 °C.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Thermostat {
    is_heat: bool,
}

impl Thermostat {
    /// A thermostat whose heater is off.
    pub fn new() -> Self {
        Self { is_heat: false }
    }

    /// Whether the heater must be on.
    pub fn heater_on(&self) -> bool {
        self.is_heat
    }

    /// Gives the thermostat the latest temperature of the battery, in hundredths of a degree
    /// Celsius, and lets it decide.
    ///
    /// To be called at every cycle that has a valid measure. Without a measure, the thermostat
    /// is not called and keeps its state.
    pub fn update(&mut self, bat_temp: i16) {
        if bat_temp < 500 {
            self.is_heat = true;
        }
        if bat_temp > 1000 {
            self.is_heat = false;
        }
    }
}
