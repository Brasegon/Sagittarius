//! PCDU (Power Conditioning and Distribution Unit): measures the battery, drives the switches.
//!
//! Format: `docs/icd.md`, section "PCDU". Multi-byte fields are big-endian.

use crate::lockstep::Payload;

pub const PCDU_ADDRESS: u8 = 0x10; // Address of the PCDU on the equipment bus

const GET_HK: u8 = 0x01; // Get PCDU data
const SET_SWITCHES: u8 = 0x02; // Set switches state

pub const SWITCH_BAT_HEATER: u8 = 1 << 0; // Battery heater switch

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcduRequest {
    GetHk,
    SetSwitches { switches: u8 },
}

impl PcduRequest {
    pub fn encode(&self) -> Payload {
        match *self {
            Self::GetHk => Payload::from_array([GET_HK]),
            Self::SetSwitches { switches } => Payload::from_array([SET_SWITCHES, switches]),
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        match *bytes {
            [GET_HK] => Some(Self::GetHk),
            [SET_SWITCHES, value] => Some(Self::SetSwitches { switches: value }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcduHk {
    pub bat_v: u16,    // Battery voltage in mV
    pub bat_temp: i16, // Battery temperature in 0.01 °C
    pub switches: u8,  // Switches state
}

impl PcduHk {
    pub const LEN: usize = 5;
    pub fn encode(&self) -> [u8; Self::LEN] {
        let bat_v_bytes: [u8; 2] = self.bat_v.to_be_bytes();
        let bat_temp_bytes: [u8; 2] = self.bat_temp.to_be_bytes();
        [
            bat_v_bytes[0],
            bat_v_bytes[1],
            bat_temp_bytes[0],
            bat_temp_bytes[1],
            self.switches,
        ]
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let bytes_decoded: [u8; Self::LEN] = match bytes.try_into() {
            Ok(array) => array,
            Err(_) => return None,
        };

        Some(Self {
            bat_v: u16::from_be_bytes([bytes_decoded[0], bytes_decoded[1]]),
            bat_temp: i16::from_be_bytes([bytes_decoded[2], bytes_decoded[3]]),
            switches: bytes_decoded[4],
        })
    }
}

#[cfg(test)]
mod tests;
