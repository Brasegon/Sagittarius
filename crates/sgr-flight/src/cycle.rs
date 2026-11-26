//! One cycle of the flight software, minimal version: read the PCDU, let the thermostat decide,
//! command the switches.
//!
//! A single function plays the whole cycle. The scheduler, the `Subsystem` trait and the
//! internal bus of `docs/conception.md` will replace it later in phase 1.
//!
//! Rules of the cycle: `docs/decisions/0004-boucle-minimale.md`.

use sgr_flight_core::platform::{CycleSync, EquipmentBus};

use crate::{
    drivers::{
        DriverError,
        pcdu::{PcduHk, SWITCH_BAT_HEATER, read_hk, set_switches},
    },
    thermal::Thermostat,
};

/// What happened with the PCDU during one cycle.
///
/// An error does not stop the cycle: it is kept here, for the caller to report. Later, it will
/// feed the event service and the housekeeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleReport {
    /// Housekeeping read at the start of the cycle, or why it could not be read.
    pub acquisition: Result<PcduHk, DriverError>,
    /// Outcome of the switch command sent at the end of the cycle.
    pub command: Result<(), DriverError>,
}

/// Plays one cycle of the flight software, from the tick to the end-of-cycle signal.
///
/// In order: waits for the tick, reads the housekeeping of the PCDU, gives the temperature of
/// the battery to the thermostat, commands the switches, signals the end of the cycle.
///
/// The cycle always goes to its end, whatever fails: without the end-of-cycle signal, the
/// simulator would wait forever. Without a valid measure, the thermostat is left alone and keeps
/// its last command. The state of the switches is sent at every cycle, even unchanged: if the
/// PCDU has restarted, one cycle is enough to restore it.
pub fn run_cycle<P: CycleSync + EquipmentBus>(
    platform: &mut P,
    thermostat: &mut Thermostat,
) -> CycleReport {
    platform.wait_tick();
    let acquisition = read_hk(platform);

    if let Ok(hk) = acquisition {
        thermostat.update(hk.bat_temp);
    }

    let switches = if thermostat.heater_on() {
        SWITCH_BAT_HEATER
    } else {
        0
    };

    let command = set_switches(platform, switches);

    platform.cycle_done();

    CycleReport {
        acquisition,
        command,
    }
}
