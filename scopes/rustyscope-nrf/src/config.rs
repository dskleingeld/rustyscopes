use arrayvec::ArrayVec;
use rustyscope_traits::{ConfigAction, ConfigErr};
use crate::hal::gpio;
use crate::hal::pac;
use crate::Mutex;
use core::ops::DerefMut;

pub enum AdcPin {
    P0_31(gpio::p0::P0_31<gpio::Disconnected>),
    P0_02(gpio::p0::P0_02<gpio::Disconnected>),
}

struct AdcPins {
    p0_31: Option<gpio::p0::P0_31<gpio::Disconnected>>,
    p0_02: Option<gpio::p0::P0_02<gpio::Disconnected>>,
}

pub struct InnerConfig {
    pub analog_enabled: ArrayVec<AdcPin, 8>,
    analog_available: AdcPins,
    // resolution: u8,
}

pub struct Config (pub Mutex<InnerConfig>);

impl Config {
    pub fn from_gpios(p0: pac::P0) -> Self {
        Self(Mutex::new(InnerConfig::from_gpios(p0), true))
    }
    pub async fn apply(&self, change: ConfigAction) -> Result<(), ConfigErr> {
        let mut guard = self.0.lock().await;
        let config = guard.deref_mut();
        config.apply(change)
    }
    // pub async fn analog_enabled<'a>(&'a self) { 
    //     let mut guard = self.0.lock().await;
    //     let config = guard.deref_mut();
    //     config.analog_enabled()
    // }
}

impl InnerConfig {
    pub fn from_gpios(p0: pac::P0) -> Self {
        use crate::hal::gpio::p0::Parts;
        let gpios = Parts::new(p0);

        Self {
            analog_available: AdcPins {
                p0_02: Some(gpios.p0_02),
                p0_31: Some(gpios.p0_31),
            },
            analog_enabled: ArrayVec::new(),
            // resolution: 12,
        }
    }

    pub fn apply(&mut self, change: ConfigAction) -> Result<(), ConfigErr> {
        use ConfigAction::*;

        match change {
            ResetPins => {
                for p in self.analog_enabled.drain(..) {
                    match p {
                        AdcPin::P0_02(p02) => self.analog_available.p0_02 = Some(p02),
                        AdcPin::P0_31(p31) => self.analog_available.p0_31 = Some(p31),
                    }
                }
            }
            DigitalPins(_pin) => Err(ConfigErr::Unimplemented)?,
            AnalogPins(pin) => {
                let adc_pin = match pin {
                    2 => self // TODO turn into macro
                        .analog_available
                        .p0_02
                        .take()
                        .ok_or(ConfigErr::PinTaken(pin))
                        .map(|p| AdcPin::P0_02(p))?,
                    31 => self
                        .analog_available
                        .p0_31
                        .take()
                        .ok_or(ConfigErr::PinTaken(pin))
                        .map(|p| AdcPin::P0_31(p))?,
                    _ => return Err(ConfigErr::InvalidPin(pin)),
                };
                self.analog_enabled.push(adc_pin);
            }
            AnalogRate(_rate) => Err(ConfigErr::Unimplemented)?,
        }

        Ok(())
    }
}
