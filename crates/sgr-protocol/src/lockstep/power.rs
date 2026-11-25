//! Power subsystem: battery and, later, solar arrays.
//!
//! Its measurements travel inside a [`SensorFrame`](super::SensorFrame). Inventory and wire
//! format: `docs/icd.md`.

/// Power measurements of one cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerSensors {
    /// Battery bus voltage `BAT_V`, in mV: `28_000` is 28 V.
    pub bat_v: u16,
}

impl PowerSensors {
    /// Size of this group on the wire, in bytes.
    pub(super) const LEN: usize = 2;

    /// Encodes the group into its wire format. Cannot fail.
    pub(super) fn encode(&self) -> [u8; Self::LEN] {
        self.bat_v.to_be_bytes()
    }

    /// Decodes the group. Cannot fail: every bit pattern is a valid measurement.
    pub(super) fn decode(bytes: [u8; Self::LEN]) -> Self {
        Self {
            bat_v: u16::from_be_bytes(bytes),
        }
    }
}
