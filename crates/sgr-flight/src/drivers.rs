//! Equipment drivers: one module per equipment of the satellite.
//!
//! A driver questions its equipment through the `EquipmentBus` trait of the platform, turns the
//! bytes into measures and reports an equipment that does not answer properly. Drivers are the
//! only code that uses the equipment bus, and the only code of this crate that uses
//! `sgr_protocol` (its `equipment` module only).
//!
//! Shape of the drivers: `docs/decisions/0003-pilotes-d-equipements.md`.

pub mod pcdu;

/// Why a driver could not get a usable answer from its equipment.
///
/// Common to every driver, so that the FDIR can count the failures of each equipment the same
/// way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// The equipment stayed silent: it is off, it has failed, or it did not understand the
    /// request.
    NoResponse,
    /// The equipment answered, but not with what the driver expected: its response has the wrong
    /// length, or does not fit in the buffer of the driver.
    InvalidResponse,
}
