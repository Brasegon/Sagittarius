//! Flight software equipment drivers and subsystems.
//!
//! - Drivers ([`drivers`]): one per equipment. Only they talk to the equipment bus.
//! - Subsystems: power, thermal, propulsion, attitude, modes and FDIR. They only talk to each
//!   other through the internal bus.

// `no_std` everywhere except in unit tests, which may use `std` (Vec, println!...).
#![cfg_attr(not(test), no_std)]

pub mod drivers;
pub mod thermal;
