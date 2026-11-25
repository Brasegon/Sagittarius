//! One cycle of the flight software, played against a fake platform.
//!
//! The fake platform is at once the cadence of the cycle and the equipment bus, like the
//! lockstep platform, where one socket carries the `Tick`, the bus exchanges and the
//! `CycleDone`. Behind its bus sits a small PCDU model.

use sgr_flight::cycle::{CycleReport, run_cycle};
use sgr_flight::drivers::DriverError;
use sgr_flight::drivers::pcdu::{PcduHk, SWITCH_BAT_HEATER};
use sgr_flight::thermal::Thermostat;
use sgr_flight_core::platform::{BusError, CycleSync, EquipmentBus};
use sgr_protocol::equipment::pcdu::{PCDU_ADDRESS, PcduRequest};

// --- The fake platform ---

/// What the platform was asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Call {
    WaitTick,
    GetHk,
    SetSwitches(u8),
    CycleDone,
}

/// How the PCDU model behaves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Health {
    /// It answers every request.
    Nominal,
    /// It answers nothing.
    Silent,
    /// It obeys SET_SWITCHES, but answers GET_HK with a truncated housekeeping.
    GarbledHk,
}

struct FakePlatform {
    /// What the PCDU measures, and the real state of its switches.
    hk: PcduHk,
    health: Health,
    /// Every call received, in order.
    calls: Vec<Call>,
}

impl FakePlatform {
    fn with_battery_at(bat_temp: i16) -> Self {
        Self {
            hk: PcduHk {
                bat_v: 28_000,
                bat_temp,
                switches: 0,
            },
            health: Health::Nominal,
            calls: Vec::new(),
        }
    }

    fn heater_is_on(&self) -> bool {
        self.hk.switches & SWITCH_BAT_HEATER != 0
    }
}

impl CycleSync for FakePlatform {
    fn wait_tick(&mut self) {
        self.calls.push(Call::WaitTick);
    }

    fn cycle_done(&mut self) {
        self.calls.push(Call::CycleDone);
    }
}

impl EquipmentBus for FakePlatform {
    fn transfer(
        &mut self,
        address: u8,
        request: &[u8],
        response: &mut [u8],
    ) -> Result<usize, BusError> {
        if address != PCDU_ADDRESS {
            return Err(BusError::NoResponse);
        }

        match PcduRequest::decode(request) {
            Some(PcduRequest::GetHk) => {
                self.calls.push(Call::GetHk);
                let bytes = self.hk.encode();
                match self.health {
                    Health::Nominal => answer(&bytes, response),
                    Health::GarbledHk => answer(&bytes[..3], response),
                    Health::Silent => Err(BusError::NoResponse),
                }
            }
            Some(PcduRequest::SetSwitches { switches }) => {
                self.calls.push(Call::SetSwitches(switches));
                if self.health == Health::Silent {
                    return Err(BusError::NoResponse);
                }
                self.hk.switches = switches;
                Ok(0)
            }
            None => Err(BusError::NoResponse),
        }
    }
}

/// Copies an answer into the buffer of the caller, as a bus does.
fn answer(bytes: &[u8], response: &mut [u8]) -> Result<usize, BusError> {
    if bytes.len() > response.len() {
        return Err(BusError::ResponseTooLong);
    }
    response[..bytes.len()].copy_from_slice(bytes);
    Ok(bytes.len())
}

// --- A nominal cycle ---

#[test]
fn cycle_does_its_steps_in_order() {
    let mut platform = FakePlatform::with_battery_at(-525);
    let mut thermostat = Thermostat::new();

    run_cycle(&mut platform, &mut thermostat);

    assert_eq!(
        platform.calls,
        [
            Call::WaitTick,
            Call::GetHk,
            Call::SetSwitches(SWITCH_BAT_HEATER),
            Call::CycleDone,
        ]
    );
}

#[test]
fn cycle_turns_the_heater_on_when_the_battery_is_cold() {
    let mut platform = FakePlatform::with_battery_at(-525); // -5.25 °C
    let mut thermostat = Thermostat::new();

    let report = run_cycle(&mut platform, &mut thermostat);

    assert!(platform.heater_is_on());
    // The report carries what was read at the start of the cycle, before the command.
    assert_eq!(
        report,
        CycleReport {
            acquisition: Ok(PcduHk {
                bat_v: 28_000,
                bat_temp: -525,
                switches: 0,
            }),
            command: Ok(()),
        }
    );
}

#[test]
fn cycle_leaves_the_heater_off_when_the_battery_is_warm() {
    let mut platform = FakePlatform::with_battery_at(2_150); // +21.50 °C
    let mut thermostat = Thermostat::new();

    let report = run_cycle(&mut platform, &mut thermostat);

    assert!(!platform.heater_is_on());
    assert_eq!(report.command, Ok(()));
    // The command is sent even though nothing changes.
    assert_eq!(platform.calls[2], Call::SetSwitches(0));
}

#[test]
fn cycle_keeps_the_heater_state_from_one_cycle_to_the_next() {
    let mut platform = FakePlatform::with_battery_at(0);
    let mut thermostat = Thermostat::new();

    // The battery warms up, then cools down a little: the hysteresis of the thermostat must
    // survive from one cycle to the next.
    for (bat_temp, expected) in [(-525, true), (700, true), (1_200, false), (700, false)] {
        platform.hk.bat_temp = bat_temp;

        run_cycle(&mut platform, &mut thermostat);

        assert_eq!(platform.heater_is_on(), expected, "at {bat_temp}");
    }
}

#[test]
fn cycle_restores_the_heater_after_a_reset_of_the_pcdu() {
    let mut platform = FakePlatform::with_battery_at(-525);
    let mut thermostat = Thermostat::new();
    run_cycle(&mut platform, &mut thermostat);
    assert!(platform.heater_is_on());

    // The PCDU restarts and opens every switch, while the battery is between the thresholds.
    platform.hk.switches = 0;
    platform.hk.bat_temp = 700;

    // The state command is sent again at every cycle: one cycle is enough to restore it.
    run_cycle(&mut platform, &mut thermostat);
    assert!(platform.heater_is_on());
}

// --- A cycle that goes wrong ---

#[test]
fn cycle_ends_even_when_the_pcdu_is_silent() {
    let mut platform = FakePlatform::with_battery_at(-525);
    platform.health = Health::Silent;
    let mut thermostat = Thermostat::new();

    let report = run_cycle(&mut platform, &mut thermostat);

    assert_eq!(
        report,
        CycleReport {
            acquisition: Err(DriverError::NoResponse),
            command: Err(DriverError::NoResponse),
        }
    );
    // Without `CycleDone`, the simulator would wait forever. The command is still attempted.
    assert_eq!(
        platform.calls,
        [
            Call::WaitTick,
            Call::GetHk,
            Call::SetSwitches(0),
            Call::CycleDone,
        ]
    );
}

#[test]
fn cycle_holds_the_heater_command_when_the_measure_is_missing() {
    let mut platform = FakePlatform::with_battery_at(-525);
    let mut thermostat = Thermostat::new();
    run_cycle(&mut platform, &mut thermostat);
    assert!(thermostat.heater_on());

    // The battery is warm now, but the flight software cannot know it: the housekeeping comes
    // back truncated. The thermostat keeps its last command, and the command is still sent.
    platform.hk.bat_temp = 2_150;
    platform.health = Health::GarbledHk;
    platform.calls.clear();

    let report = run_cycle(&mut platform, &mut thermostat);

    assert_eq!(report.acquisition, Err(DriverError::InvalidResponse));
    assert_eq!(report.command, Ok(()));
    assert!(thermostat.heater_on());
    assert_eq!(platform.calls[2], Call::SetSwitches(SWITCH_BAT_HEATER));

    // The measure comes back: the thermostat catches up.
    platform.health = Health::Nominal;

    let report = run_cycle(&mut platform, &mut thermostat);

    assert_eq!(report.command, Ok(()));
    assert!(!thermostat.heater_on());
    assert!(!platform.heater_is_on());
}
