//! Driver of the PCDU (Power Conditioning and Distribution Unit).
//!
//! One function per command of the ICD (`docs/icd.md`, section "PCDU"). Each one does a single
//! exchange on the bus: it never retries by itself.

use sgr_flight_core::platform::{BusError, EquipmentBus};
use sgr_protocol::equipment::pcdu::{self, PcduRequest};
pub use sgr_protocol::equipment::pcdu::{PcduHk, SWITCH_BAT_HEATER};

use crate::drivers::DriverError;

/// Asks the PCDU for its housekeeping: the voltage and the temperature of the battery, and the
/// real state of the switches.
///
/// # Errors
///
/// - [`DriverError::NoResponse`] if the PCDU stays silent.
/// - [`DriverError::InvalidResponse`] if its answer is not a housekeeping of [`PcduHk::LEN`]
///   bytes.
pub fn read_hk<B: EquipmentBus>(bus: &mut B) -> Result<PcduHk, DriverError> {
    let mut buffer = [0u8; PcduHk::LEN];

    let len = exchange(bus, PcduRequest::GetHk, &mut buffer)?;

    if len != PcduHk::LEN {
        return Err(DriverError::InvalidResponse);
    }

    let hk = match PcduHk::decode(&buffer) {
        None => return Err(DriverError::InvalidResponse),
        Some(hk) => hk,
    };

    Ok(hk)
}

/// Commands the state of every switch at once: one bit per switch, see the `SWITCH_*` constants
/// of `sgr_protocol::equipment::pcdu`.
///
/// `Ok` only means that the PCDU acknowledged the command. Reading the housekeeping again tells
/// whether the switches really moved.
///
/// # Errors
///
/// - [`DriverError::NoResponse`] if the PCDU stays silent.
/// - [`DriverError::InvalidResponse`] if it answers with anything but an empty response.
pub fn set_switches<B: EquipmentBus>(bus: &mut B, switches: u8) -> Result<(), DriverError> {
    let mut buffer = [0u8; PcduHk::LEN];

    let len = exchange(bus, PcduRequest::SetSwitches { switches }, &mut buffer)?;

    if len != 0 {
        return Err(DriverError::InvalidResponse);
    }
    Ok(())
}

/// One exchange with the PCDU: sends `request`, copies the answer at the start of `response`
/// and returns its length.
///
/// This is the only place where the errors of the bus are translated into what they mean for
/// the equipment.
fn exchange<B: EquipmentBus>(
    bus: &mut B,
    request: PcduRequest,
    response: &mut [u8],
) -> Result<usize, DriverError> {
    match bus.transfer(pcdu::PCDU_ADDRESS, &request.encode(), response) {
        Ok(len) => Ok(len),
        Err(BusError::NoResponse) => Err(DriverError::NoResponse),
        Err(BusError::ResponseTooLong) => Err(DriverError::InvalidResponse),
    }
}
