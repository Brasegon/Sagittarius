//! Thermal subsystem: temperatures and heaters.
//!
//! Its measurements travel inside a [`SensorFrame`](super::SensorFrame). Inventory and wire
//! format: `docs/icd.md`.
//!
//! The heater switches are not here: every `SWITCH_*` bit is declared in the parent module,
//! next to [`ActuatorFrame`](super::ActuatorFrame), because they all share one byte.

/// Thermal measurements of one cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermalSensors {
    /// Battery temperature `BAT_TEMP`, in hundredths of a °C: `-525` is -5.25 °C.
    pub bat_temp: i16,
}

impl ThermalSensors {
    /// Size of this group on the wire, in bytes.
    pub(super) const LEN: usize = 2;

    /// Encodes the group into its wire format. Cannot fail.
    pub(super) fn encode(&self) -> [u8; Self::LEN] {
        self.bat_temp.to_be_bytes()
    }

    /// Decodes the group. Cannot fail: every bit pattern is a valid measurement.
    pub(super) fn decode(bytes: [u8; Self::LEN]) -> Self {
        Self {
            bat_temp: i16::from_be_bytes(bytes),
        }
    }
}
