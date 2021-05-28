#![feature(array_methods)]
#![cfg_attr(not(test), no_std)]
use serde::{Deserialize, Serialize};
use core::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug)]
pub enum Mode {
    Idle,
    Continues(SampleKind),
    Burst(SampleKind),
}

#[derive(Serialize, Deserialize, Debug, defmt::Format, Copy, Clone, PartialEq)]
pub enum SampleKind {
    Digital,
    Analog,
}

#[derive(Serialize, Deserialize, Debug, defmt::Format, Copy, Clone, PartialEq)]
pub enum ConfigErr {
    UnavailibleSampler(Sampler),
    PinTaken(Pin),
    InvalidPin(Pin),
    InvalidRate(u32),
    Unimplemented,
    CommunicationProblem,
}

pub type Pin = u8;
pub type Sampler = u8;
#[derive(Serialize, Deserialize, Debug, defmt::Format, Copy, Clone, PartialEq)]
pub enum ConfigAction {
    ResetPins,
    /// add pin to measure
    DigitalPins(Pin),
    /// add pin to measure
    AnalogPins(Pin),
    AnalogRate(u32),
}

#[derive(Serialize, Deserialize, Debug, defmt::Format, PartialEq)]
pub enum Command {
    /// Stop continues sampling
    Stop,
    /// start sampling while sending back data
    Continues(SampleKind),
    /// start to sample as fast as possible, this can not be interrupted
    /// once stated as the device stops listening to uart
    /// until it is done
    Burst(SampleKind),
    /// configure sampling
    Config(ConfigAction),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Reply {
    Done(u32),
    Data(u32),
    Err(ConfigErr),
    Ok,
}

impl Reply {
    pub const SIZE: usize = 6;
    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        postcard::to_slice(self, &mut buf).unwrap();
        buf
    }
}

impl TryFrom<&[u8; Self::SIZE]> for Reply {
    type Error = postcard::Error;

    fn try_from(s: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        postcard::from_bytes(s)
    }
}

impl TryFrom<&[u8; Self::SIZE]> for Command {
    type Error = postcard::Error;

    fn try_from(s: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        postcard::from_bytes(s)
    }
}

impl Command {
    pub const SIZE: usize = 6;
    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        postcard::to_slice(self, &mut buf).unwrap();
        buf
    }
}

#[allow(dead_code)]
pub struct Abilities {
    /// pins that can be configured to
    /// listen on
    pub adc_pins: &'static [Pin],
    /// resolution in bits
    pub adc_res: &'static [u8],
    /// voltage reference options
    pub adc_ref: &'static [&'static str],
}

#[cfg(test)]
mod tests {
    use super::*;

    mod commands {
        use super::*;

        const COMMANDS: [Command; 5] = [
            Command::Stop,
            Command::Continues(SampleKind::Analog),
            Command::Burst(SampleKind::Digital),
            Command::Config(ConfigAction::AnalogPins(0u8)),
            Command::Config(ConfigAction::AnalogRate(0u32)),
        ];

        #[test]
        fn not_to_short() {
            for cmd in &COMMANDS {
                let _ = cmd.serialize();
            }
        }

        #[test]
        fn serialize_deserialize() {
            for cmd in &COMMANDS {
                let buf = cmd.serialize();
                let deserialized = Command::try_from(&buf).unwrap();
                assert_eq!(&deserialized, cmd);
            }
        }
    }

    mod reply {
        use super::*;

        const REPLIES: [Reply; 3] = [
            Reply::Ok,
            Reply::Err(ConfigErr::InvalidRate(u32::MAX)),
            Reply::Data(u32::MAX),
        ];

        #[test]
        fn serialize() {
            for rply in &REPLIES {
                let _ = rply.serialize();
            }
        }

        #[test]
        fn serialize_deserialize() {
            for rply in &REPLIES {
                let buf = rply.serialize();
                let deserialized = Reply::try_from(&buf).unwrap();
                assert_eq!(&deserialized, rply);
            }
        }
    }
}
