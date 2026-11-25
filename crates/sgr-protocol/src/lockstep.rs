//! Lockstep frames exchanged between the simulator and the flight software.
//!
//! Wire format: `docs/icd.md`, section "Interface lockstep". Multi-byte fields are big-endian.
//!
//! The measurements are grouped by subsystem, one file each ([`power`], [`thermal`]). This
//! module owns the frames themselves: the header, and the place of each group in the frame.

pub mod power;
pub mod thermal;

pub use power::PowerSensors;
pub use thermal::ThermalSensors;

/// Version of the lockstep interface, carried in byte 0 of every frame.
///
/// Bumped on every incompatible change of a frame. Never 0, so that an all-zero buffer is
/// not a valid frame.
pub const LOCKSTEP_VERSION: u8 = 1;

// Every `SWITCH_*` bit is declared here, whatever its subsystem: they all share the single
// `switches` byte, so they are kept together to see at a glance that no bit is used twice.

/// Bit of [`ActuatorFrame::switches`] that drives the battery heater `BAT_HEATER` (1 = on).
pub const SWITCH_BAT_HEATER: u8 = 1 << 0;

/// Sensor measurements, sent by the simulator at the start of each cycle.
///
/// 10 bytes on the wire: a 6-byte header (version, frame type, tick), then one group of
/// measurements per subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorFrame {
    /// Cycle number (bytes 2 to 5). On-board time is `tick` x 100 ms.
    pub tick: u32,
    /// Power measurements (bytes 6 and 7).
    pub power: PowerSensors,
    /// Thermal measurements (bytes 8 and 9).
    pub thermal: ThermalSensors,
}

/// Actuator commands, sent back by the flight software for the same cycle.
///
/// 7 bytes on the wire: a 6-byte header (version, frame type, tick), then the switches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActuatorFrame {
    /// Cycle number (bytes 2 to 5), copied from the [`SensorFrame`] being answered.
    pub tick: u32,
    /// Wanted state of every switch, one bit each (byte 6): see the `SWITCH_*` constants.
    /// Unassigned bits are reserved: sent as 0, ignored on reception.
    pub switches: u8,
}

/// Why a received buffer is not a valid frame (ICD, "Règles de réception").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The buffer does not have the exact length of the expected frame.
    WrongLength,
    /// Byte 0 is not [`LOCKSTEP_VERSION`].
    UnsupportedVersion,
    /// Byte 1 is not the frame type expected in this direction.
    UnexpectedFrameType,
}

/// Value of the frame type, carried in byte 1 of every frame.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameType {
    Sensor = 1,
    Actuator = 2,
}

/// Checks the length, the version and the frame type of a received buffer.
///
/// On success, returns the bytes as an array of the exact frame size `N`: the caller can then
/// index it with constants, which the compiler checks.
fn check_header<const N: usize>(
    bytes: &[u8],
    frame_type: FrameType,
) -> Result<[u8; N], DecodeError> {
    // Compile-time guard: `frame[0]` and `frame[1]` below need at least 2 bytes.
    const { assert!(N >= 2) };

    let frame: [u8; N] = match bytes.try_into() {
        Ok(array) => array,
        Err(_) => return Err(DecodeError::WrongLength),
    };

    if frame[0] != LOCKSTEP_VERSION {
        return Err(DecodeError::UnsupportedVersion);
    }

    if frame[1] != frame_type as u8 {
        return Err(DecodeError::UnexpectedFrameType);
    }

    Ok(frame)
}

impl SensorFrame {
    /// Size of the frame on the wire, in bytes.
    pub const LEN: usize = 10;

    /// Encodes the frame into its wire format. Cannot fail.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let tick: [u8; 4] = self.tick.to_be_bytes();
        let power: [u8; 2] = self.power.encode();
        let thermal: [u8; 2] = self.thermal.encode();
        [
            LOCKSTEP_VERSION,
            FrameType::Sensor as u8,
            tick[0],
            tick[1],
            tick[2],
            tick[3],
            power[0],
            power[1],
            thermal[0],
            thermal[1],
        ]
    }

    /// Decodes a received buffer, or says why it is not a valid `SensorFrame`.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let frame: [u8; Self::LEN] = check_header(bytes, FrameType::Sensor)?;

        Ok(Self {
            tick: u32::from_be_bytes([frame[2], frame[3], frame[4], frame[5]]),
            power: PowerSensors::decode([frame[6], frame[7]]),
            thermal: ThermalSensors::decode([frame[8], frame[9]]),
        })
    }
}

impl ActuatorFrame {
    /// Size of the frame on the wire, in bytes.
    pub const LEN: usize = 7;

    /// Encodes the frame into its wire format. Cannot fail.
    pub fn encode(&self) -> [u8; Self::LEN] {
        let tick: [u8; 4] = self.tick.to_be_bytes();
        let switches: [u8; 1] = self.switches.to_be_bytes();
        [
            LOCKSTEP_VERSION,
            FrameType::Actuator as u8,
            tick[0],
            tick[1],
            tick[2],
            tick[3],
            switches[0],
        ]
    }

    /// Decodes a received buffer, or says why it is not a valid `ActuatorFrame`.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let frame: [u8; Self::LEN] = check_header(bytes, FrameType::Actuator)?;

        Ok(Self {
            tick: u32::from_be_bytes([frame[2], frame[3], frame[4], frame[5]]),
            switches: u8::from_be_bytes([frame[6]]),
        })
    }
}

#[cfg(test)]
mod tests;
