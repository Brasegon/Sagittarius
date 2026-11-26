//! Model of the PCDU (Power Conditioning and Distribution Unit).
//!
//! It keeps the real state of the switches and answers the two commands of the ICD
//! (`docs/icd.md`, section "PCDU"). Its measures come from the world: it is the one that turns
//! a temperature in degrees into hundredths of a degree on the wire.
//!
//! Rules of the model: `docs/decisions/0005-simulateur-minimal.md`.

use sgr_protocol::{
    equipment::pcdu::{
        PcduHk,
        PcduRequest::{self},
        SWITCH_BAT_HEATER,
    },
    lockstep::Payload,
};

use crate::world::World;

/// The PCDU as the simulator sees it: the real state of its switches.
///
/// It stores no measure: it reads the world again at every request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pcdu {
    /// State of the switches, one bit per switch (`SWITCH_*` in the ICD). The reserved bits
    /// always stay at 0.
    switches: u8,
}

impl Pcdu {
    /// A PCDU that has just been powered: every switch is open.
    pub fn new() -> Self {
        Self { switches: 0 }
    }

    /// Whether the battery heater is powered. The simulator passes it on to the world at every
    /// step.
    pub fn heater_on(&self) -> bool {
        self.switches & SWITCH_BAT_HEATER != 0
    }

    /// Answers one request received on the equipment bus, or stays silent.
    ///
    /// - `GET_HK`: measures `world` as it is now and returns the encoded [`PcduHk`]. Each
    ///   measure is rounded to the nearest step of its unit (1 mV, 0.01 °C), then saturated at
    ///   the bounds of its type.
    /// - `SET_SWITCHES`: replaces the state of every switch at once and returns an empty
    ///   response. The reserved bits are ignored.
    /// - Anything else (unknown command, wrong length): `None`, and nothing changes. On the
    ///   bus, `None` is a silence.
    pub fn handle(&mut self, world: &World, request: &[u8]) -> Option<Payload> {
        let decode = match PcduRequest::decode(request) {
            None => None,
            Some(PcduRequest::GetHk) => {
                let hk = PcduHk {
                    bat_v: (world.bat_v * 1000.0).round() as u16,
                    bat_temp: (world.bat_temp * 100.0).round() as i16,
                    switches: self.switches,
                };
                Some(Payload::from_array(hk.encode()))
            }
            Some(PcduRequest::SetSwitches { switches }) => {
                self.switches = switches & SWITCH_BAT_HEATER;
                Some(Payload::new())
            }
        };
        decode
    }
}
