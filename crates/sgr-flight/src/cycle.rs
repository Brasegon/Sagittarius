//! One cycle of the flight software, minimal version: read the PCDU, let the thermostat decide,
//! command the switches.
//!
//! A single function plays the whole cycle. The scheduler, the `Subsystem` trait and the
//! internal bus of `docs/conception.md` will replace it later in phase 1.
//!
//! Rules of the cycle: `docs/decisions/0004-boucle-minimale.md`.
