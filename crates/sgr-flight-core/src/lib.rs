//! Flight software core: platform traits and core services
//! (scheduler, internal bus, parameters, events, TC/TM, telemetry store).

// `no_std` everywhere except in unit tests, which may use `std` (Vec, println!...).
#![cfg_attr(not(test), no_std)]
