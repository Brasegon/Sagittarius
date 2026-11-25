//! Flight software subsystems: power, thermal, propulsion, attitude, modes and FDIR.
//!
//! Subsystems only talk to each other through the internal bus.

// `no_std` everywhere except in unit tests, which may use `std` (Vec, println!...).
#![cfg_attr(not(test), no_std)]
