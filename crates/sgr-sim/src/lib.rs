//! Environment simulator: orbit, power, thermal, propulsion and radio channel.
//!
//! A library so that end-to-end tests can step the world in-process, without sockets.
//! The `sgr-sim` binary wraps it with the UDP transport.
//!
//! - The world ([`world`]): the physics, in physical units.
//! - The equipment ([`equipment`]): one model per equipment of the satellite. Each one measures
//!   the world and answers the requests of the flight software.
//! - The simulator ([`sim`]): puts them together behind the equipment bus, and counts the ticks.
//!
//! Shape of the simulator: `docs/decisions/0005-simulateur-minimal.md`.

pub mod equipment;
pub mod sim;
pub mod world;
