//! Message formats shared by the flight software and the simulator.
//!
//! `docs/icd.md` is the source of truth: change the ICD first, then this crate.

// `no_std` everywhere except in unit tests, which may use `std` (Vec, println!...).
#![cfg_attr(not(test), no_std)]

pub mod lockstep;
