//! Equipment of the satellite: what each one understands and what it answers.
//!
//! These formats travel as the payload of the bus frames of [`crate::lockstep`], which carries
//! them without reading them. Each one is shared by the driver of the equipment (flight
//! software) and by its model (simulator).
//!
//! Source of truth: `docs/icd.md`, section "Équipements".

pub mod pcdu;
