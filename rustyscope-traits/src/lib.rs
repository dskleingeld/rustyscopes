#![no_std]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Mode {
    Idle,
    Continues(SampleKind),
    Burst(SampleKind),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SampleKind {
    Digital,
    Analog,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response<'a> {
    Ok,
    Err(ConfigErr),
    Data(&'a[u8])
}

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub enum ConfigAction {
    ResetPins,
    /// add pin to listen to
    DigitalPins(Pin),
    /// add pin to listen to
    AnalogPins(Pin),
    AnalogRate(u32),
}

#[derive(Serialize, Deserialize, Debug)]
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
