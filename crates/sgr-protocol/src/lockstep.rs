//! Lockstep frames exchanged between the simulator and the flight software.
//!
//! Wire format: `docs/icd.md`, section "Interface lockstep". Multi-byte fields are big-endian.

/// Version of the lockstep interface, carried in byte 0 of every frame.
///
/// Bumped on every incompatible change of a frame. Never 0, so that an all-zero buffer is
/// not a valid frame.
pub const LOCKSTEP_VERSION: u8 = 2;

/// Largest payload a bus frame can carry, in bytes.
pub const MAX_PAYLOAD: usize = 32;

/// Size of the fixed part of a bus frame: the 6-byte header, then 1 byte (address or status).
const BUS_FIXED_LEN: usize = 7;

/// Largest bus frame, in bytes.
pub const MAX_FRAME_LEN: usize = BUS_FIXED_LEN + MAX_PAYLOAD;

/// Payload of a bus frame: up to [`MAX_PAYLOAD`] bytes, stored without heap allocation.
pub type Payload = heapless::Vec<u8, MAX_PAYLOAD>;

/// Encoded bus frame: up to [`MAX_FRAME_LEN`] bytes, stored without heap allocation.
pub type FrameBytes = heapless::Vec<u8, MAX_FRAME_LEN>;

/// Why a received buffer is not a valid frame (ICD, "Règles de réception").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The buffer is too short or too long for the expected frame.
    WrongLength,
    /// Byte 0 is not [`LOCKSTEP_VERSION`].
    UnsupportedVersion,
    /// Byte 1 is not the frame type expected in this direction.
    UnexpectedFrameType,
    /// Byte 6 of a `BusResponse` is not a known [`BusStatus`].
    InvalidStatus,
}

/// Value of the frame type, carried in byte 1 of every frame.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameType {
    Tick = 1,
    BusRequest = 2,
    BusResponse = 3,
    CycleDone = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick {
    pub tick: u32,
}

impl Tick {
    pub const LEN: usize = 6;

    pub fn encode(&self) -> [u8; Self::LEN] {
        let tick: [u8; 4] = self.tick.to_be_bytes();
        [
            LOCKSTEP_VERSION,
            FrameType::Tick as u8,
            tick[0],
            tick[1],
            tick[2],
            tick[3],
        ]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let frame: [u8; Self::LEN] = check_header(bytes, FrameType::Tick)?;

        Ok(Self {
            tick: u32::from_be_bytes([frame[2], frame[3], frame[4], frame[5]]),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleDone {
    pub tick: u32,
}

impl CycleDone {
    pub const LEN: usize = 6;

    pub fn encode(&self) -> [u8; Self::LEN] {
        let tick: [u8; 4] = self.tick.to_be_bytes();
        [
            LOCKSTEP_VERSION,
            FrameType::CycleDone as u8,
            tick[0],
            tick[1],
            tick[2],
            tick[3],
        ]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let frame: [u8; Self::LEN] = check_header(bytes, FrameType::CycleDone)?;

        Ok(Self {
            tick: u32::from_be_bytes([frame[2], frame[3], frame[4], frame[5]]),
        })
    }
}

/// Request sent by the flight software to one equipment of the bus.
///
/// 7 to 39 bytes on the wire: the 6-byte header, the address, then the payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusRequest {
    /// Cycle number (bytes 2 to 5).
    pub tick: u32,
    /// Address of the equipment being asked (byte 6).
    pub address: u8,
    /// Request, in the format of that equipment (bytes 7 and up). The bus does not read it.
    pub payload: Payload,
}

impl BusRequest {
    /// Encodes the frame into its wire format. Cannot fail.
    pub fn encode(&self) -> FrameBytes {
        let tick: [u8; 4] = self.tick.to_be_bytes();
        assemble(
            [
                LOCKSTEP_VERSION,
                FrameType::BusRequest as u8,
                tick[0],
                tick[1],
                tick[2],
                tick[3],
                self.address,
            ],
            &self.payload,
        )
    }

    /// Decodes a received buffer, or says why it is not a valid `BusRequest`.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (fixed, payload) = split(bytes, FrameType::BusRequest)?;

        Ok(Self {
            tick: u32::from_be_bytes([fixed[2], fixed[3], fixed[4], fixed[5]]),
            address: fixed[6],
            payload,
        })
    }
}

/// Outcome of a bus request, carried in byte 6 of a [`BusResponse`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusStatus {
    /// The equipment answered: the payload is its response.
    Ok = 0,
    /// The equipment stayed silent: unknown address, failure, or request it does not understand.
    NoResponse = 1,
}

/// Response of the bus to a [`BusRequest`], sent by the simulator.
///
/// 7 to 39 bytes on the wire: the 6-byte header, the status, then the payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusResponse {
    /// Cycle number (bytes 2 to 5).
    pub tick: u32,
    /// Whether the equipment answered (byte 6).
    pub status: BusStatus,
    /// Response, in the format of the equipment (bytes 7 and up). Empty if it stayed silent.
    pub payload: Payload,
}

impl BusResponse {
    /// Encodes the frame into its wire format. Cannot fail.
    pub fn encode(&self) -> FrameBytes {
        let tick: [u8; 4] = self.tick.to_be_bytes();
        assemble(
            [
                LOCKSTEP_VERSION,
                FrameType::BusResponse as u8,
                tick[0],
                tick[1],
                tick[2],
                tick[3],
                self.status as u8,
            ],
            &self.payload,
        )
    }

    /// Decodes a received buffer, or says why it is not a valid `BusResponse`.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (fixed, payload) = split(bytes, FrameType::BusResponse)?;

        // The reverse of `self.status as u8`: any other value is not a status.
        let status = match fixed[6] {
            0 => BusStatus::Ok,
            1 => BusStatus::NoResponse,
            _ => return Err(DecodeError::InvalidStatus),
        };

        Ok(Self {
            tick: u32::from_be_bytes([fixed[2], fixed[3], fixed[4], fixed[5]]),
            status,
            payload,
        })
    }
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

/// Glues the fixed bytes of a bus frame and its payload together.
fn assemble(fixed: [u8; BUS_FIXED_LEN], payload: &Payload) -> FrameBytes {
    let mut frame = FrameBytes::new();
    for byte in fixed.iter().chain(payload) {
        // Cannot be full: `MAX_FRAME_LEN` is defined as the fixed bytes plus the longest payload.
        if frame.push(*byte).is_err() {
            break;
        }
    }
    frame
}

/// Checks a received bus frame, then splits it into its fixed bytes and its payload.
fn split(
    bytes: &[u8],
    frame_type: FrameType,
) -> Result<([u8; BUS_FIXED_LEN], Payload), DecodeError> {
    let (fixed, payload): (&[u8; BUS_FIXED_LEN], &[u8]) = match bytes.split_first_chunk() {
        Some(parts) => parts,
        None => return Err(DecodeError::WrongLength),
    };

    let fixed: [u8; BUS_FIXED_LEN] = check_header(fixed, frame_type)?;

    let payload: Payload = match Payload::from_slice(payload) {
        Ok(payload) => payload,
        Err(_) => return Err(DecodeError::WrongLength),
    };

    Ok((fixed, payload))
}

#[cfg(test)]
mod tests;
