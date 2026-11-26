//! Platform traits: what the flight software needs from the machine it runs on.
//!
//! They are defined here and implemented by the platform crates: `sgr-flight-linux` today, an
//! STM32 crate later. The rest of the flight software only knows the traits, so it does not
//! change from one platform to the next. This is the role of the OSAL and the PSP in NASA's cFS.
//!
//! Shape of the traits: `docs/decisions/0002-forme-des-traits-de-plateforme.md`.

/// Source of the on-board time.
///
/// In lockstep, it follows the tick announced by the simulator. On the target, it will count the
/// periods of a hardware timer. The on-board time never comes from the clock of the host machine.
pub trait Clock {
    /// Current on-board tick.
    ///
    /// One tick is one cycle of 100 ms. Tick 0 is the separation from the launcher.
    fn tick(&self) -> u32;
}

/// Cadence of the cycle: when a cycle starts, and how the flight software says it is over.
///
/// In lockstep, these are the `Tick` and `CycleDone` frames exchanged with the simulator. On the
/// target, `wait_tick` will wait for a hardware timer.
///
/// Rules of the cycle: `docs/decisions/0004-boucle-minimale.md`.
pub trait CycleSync {
    /// Waits for the start of the next cycle.
    ///
    /// It returns nothing: the tick has a single source, [`Clock::tick`], which gives the number
    /// of the cycle that has just started.
    fn wait_tick(&mut self);

    /// Tells the platform that the flight software has finished its cycle.
    ///
    /// To be called once per cycle, whatever happened during it: in lockstep, the simulator
    /// waits for it before it moves the world forward.
    fn cycle_done(&mut self);
}

/// Why an exchange on the equipment bus failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusError {
    /// The equipment stayed silent: nobody has this address, it has failed, or it does not
    /// understand the request.
    NoResponse,
    /// The equipment answered, but its response does not fit in the buffer of the caller.
    ResponseTooLong,
}

/// Bus on which the flight software questions its equipment, one exchange at a time.
///
/// The flight software is the master of the bus: an equipment only speaks when it is asked.
/// Only the equipment drivers use this trait. It is not the internal bus, which carries messages
/// between subsystems.
pub trait EquipmentBus {
    /// Sends `request` to the equipment at `address` and waits for its response.
    ///
    /// The call returns once the equipment has answered, or once it is known to be silent. The
    /// bus carries `request` and the response without reading them: their format belongs to the
    /// equipment.
    ///
    /// On success, the response is copied at the start of `response` and its length is returned:
    /// only that many bytes are meaningful. `Ok(0)` is an empty response, from an equipment that
    /// obeyed and has nothing to say. It is not a silence.
    ///
    /// # Errors
    ///
    /// - [`BusError::NoResponse`] if the equipment stays silent.
    /// - [`BusError::ResponseTooLong`] if the response does not fit in `response`.
    ///
    /// The content of `response` is then unspecified.
    fn transfer(
        &mut self,
        address: u8,
        request: &[u8],
        response: &mut [u8],
    ) -> Result<usize, BusError>;
}
