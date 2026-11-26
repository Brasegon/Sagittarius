//! The simulator: the world and the equipment models, behind the equipment bus.
//!
//! In-memory interface of the lockstep cycle (`docs/icd.md`, "Déroulement d'un cycle"): the
//! flight software reads the tick, sends its bus requests, then the world moves forward one
//! step. The world is frozen in between. The UDP binary will wrap this interface with the
//! lockstep frames.
//!
//! Shape of the simulator: `docs/decisions/0005-simulateur-minimal.md`.
