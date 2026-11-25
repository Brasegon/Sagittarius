//! End-to-end test harness: simulator and flight software in lockstep, in one process.
//!
//! This library holds the shared harness (in-memory platform, scenario helpers).
//! The scenarios themselves (LEOP, manoeuvres, failures) go in `tests/`, one file each.
