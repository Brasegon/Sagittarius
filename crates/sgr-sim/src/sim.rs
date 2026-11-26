//! The simulator: the world and the equipment models, behind the equipment bus.
//!
//! In-memory interface of the lockstep cycle (`docs/icd.md`, "Déroulement d'un cycle"): the
//! flight software reads the tick, sends its bus requests, then the world moves forward one
//! step. The world is frozen in between. The UDP binary will wrap this interface with the
//! lockstep frames.
//!
//! Shape of the simulator: `docs/decisions/0005-simulateur-minimal.md`.

use crate::{equipment::pcdu::Pcdu, world::World};

#[derive(Debug, Clone)]
pub struct Sim {
    pub world: World,
    pub pcdu: Pcdu,
    tick: u32
}

impl Sim {
    pub fn new(world: World) -> Self {
        Self { world, pcdu: Pcdu::new(), tick: 0 }
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }
}