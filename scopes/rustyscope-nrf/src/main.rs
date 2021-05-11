#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(const_fn_trait_bound)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;

use arrayvec::ArrayVec;
use defmt::panic;
use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::{buffered_uarte::BufferedUarte, interrupt, uarte};
use futures::pin_mut;

mod description;
use description::ABILITIES;

use embedded_hal::adc;
use hal::{
    gpio,
    gpio::p0::Parts as P0Parts,
    pac::Peripherals,
    prelude::*,
    saadc::{Saadc, SaadcConfig},
};
use nrf52832_hal as hal;
use rustyscope_traits::{Command, ConfigAction, ConfigErr, Response, SampleKind};

use defmt_setup::*;

enum Mode {
    Idle,
    Continues(SampleKind),
    Burst(SampleKind),
    Err(ConfigErr),
}

enum AdcPin {
    P0_31(gpio::p0::P0_31<gpio::Disconnected>),
    P0_02(gpio::p0::P0_02<gpio::Disconnected>),
}

struct AdcPins {
    p0_31: Option<gpio::p0::P0_31<gpio::Disconnected>>,
    p0_02: Option<gpio::p0::P0_02<gpio::Disconnected>>,
}

struct Config {
    analog_enabled: ArrayVec<AdcPin, 8>,
    analog_available: AdcPins,
    resolution: u8,
}

impl Config {
    fn from(analog_available: AdcPins) -> Self {
        Self {
            analog_available,
            analog_enabled: ArrayVec::new(),
            resolution: 12,
        }
    }

    fn apply(&mut self, change: ConfigAction, adc: &mut Saadc) -> Result<(), ConfigErr> {
        use ConfigAction::*;

        match change {
            ResetPins => for p in self.analog_enabled.drain(..) {
                match p {
                    AdcPin::P0_02(p02) => self.analog_available.p0_02 = Some(p02),
                    AdcPin::P0_31(p31) => self.analog_available.p0_31 = Some(p31),
                }
            },
            DigitalPins(pin) => Err(ConfigErr::Unimplemented)?,
            AnalogPins(pin) => {
                let adc_pin = match pin {
                    2 => self
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
            AnalogRate(rate) => Err(ConfigErr::Unimplemented)?,
        }

        Ok(())
    }
}

async fn sample<PIN>(adc: &mut Saadc, pin: &mut PIN)
where
    PIN: adc::Channel<Saadc, ID = u8>,
{
    let v = adc.read(pin).unwrap();
    info!("value: {}", v);
    Timer::after(Duration::from_secs(1)).await;
}

#[embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nrf::Peripherals::take().unwrap();
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let mut tx_buffer = [0u8; 4096];
    let mut rx_buffer = [0u8; 256]; // we only get commands and those are small

    let irq = interrupt::take!(UARTE0_UART0);
    let serial = unsafe {
        BufferedUarte::new(
            p.UARTE0,
            p.TIMER0,
            p.PPI_CH0,
            p.PPI_CH1,
            irq,
            p.P0_08,
            p.P0_06,
            NoPin,
            NoPin,
            config,
            &mut rx_buffer,
            &mut tx_buffer,
        )
    };
    pin_mut!(serial);

    let board = Peripherals::take().unwrap();
    let gpios = P0Parts::new(board.P0);

    // initialize saadc interface
    let saadc_config = SaadcConfig::default();
    let mut adc = Saadc::new(board.SAADC, saadc_config);
    let analog_available = AdcPins {
        p0_02: Some(gpios.p0_02),
        p0_31: Some(gpios.p0_31),
    };
    let mut config = Config::from(analog_available);
    let mut mode = Mode::Idle;
    loop {
        // let command = recieve_commands(serial);
        let command = Command::Stop;

        mode = match command {
            Command::Stop => Mode::Idle,
            Command::Continues(s) => Mode::Continues(s),
            Command::Burst(s) => Mode::Burst(s),
            Command::Config(change) => match config.apply(change, &mut adc) {
                Result::Ok(_) => mode,
                Result::Err(e) => Mode::Err(e),
            },
        };

        use Mode::*;
        use SampleKind::*;
        /* match mode {
            Idle => (),
            Continues(Analog) => (),
            Continues(Digital) => (),
            Burst(Analog) => (),
            Burst(Digital) => (),
            Err(e) => (),
        } */
    }
}
