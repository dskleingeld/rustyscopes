
#![feature(array_methods)]
#![cfg_attr(not(test), no_std)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Mode {
    Idle,
    Continues(SampleKind),
    Burst(SampleKind),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum SampleKind {
    Digital,
    Analog,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response<'a> {
    Ok,
    Err(ConfigErr),
    Data(&'a [u8]),
}

#[derive(Serialize, Deserialize, Debug, defmt::Format, Copy, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum ConfigAction {
    ResetPins,
    /// add pin to listen to
    DigitalPins(Pin),
    /// add pin to listen to
    AnalogPins(Pin),
    AnalogRate(u32),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

impl From<&[u8; 6]> for Command {
    fn from(s: &[u8; 6]) -> Self {
        postcard::from_bytes(s).unwrap()
    }
}

impl Command {
    fn serialize<'a>(&self, buf: &'a mut [u8; 6]) {
        postcard::to_slice(self, buf).unwrap();
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

    const COMMANDS: [Command; 5] = [
        Command::Stop,
        Command::Continues(SampleKind::Analog),
        Command::Burst(SampleKind::Digital),
        Command::Config(ConfigAction::AnalogPins(0u8)),
        Command::Config(ConfigAction::AnalogRate(0u32)),
    ];

    #[test]
    fn not_to_short() {
        let mut buf = [0u8; 6];
        for cmd in &COMMANDS {
            cmd.serialize(&mut buf);
        }
    }

    #[test]
    fn serialize_deserialize() {
        for cmd in &COMMANDS {
            let mut buf = [0u8; 6];
            cmd.serialize(&mut buf);
            let deserialized =Command::from(&buf);
            assert_eq!(&deserialized, cmd);
        }
    }
}
