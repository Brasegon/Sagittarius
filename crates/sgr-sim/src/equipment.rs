//! Equipment models: one module per equipment of the satellite.
//!
//! A model is the simulator's side of an equipment. It keeps the state of the equipment,
//! measures the world and answers the requests that the flight software sends on the bus. Its
//! formats come from `sgr_protocol::equipment`, shared with the driver of the flight software.

pub mod pcdu;
