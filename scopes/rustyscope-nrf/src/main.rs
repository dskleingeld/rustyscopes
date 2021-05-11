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

struct Config {
    analog_enabled: ArrayVec<AdcPin, 8>,
    analog_available: ArrayVec<AdcPin, 2>,
    resolution: u8,
}

impl Config {
    fn from(analog_available: [AdcPin; 2]) -> Self {
        Self {
            analog_available: ArrayVec::from(analog_available),
            analog_enabled: ArrayVec::new(),
            resolution: 12,
        }
    }

    fn apply(&mut self, change: ConfigAction, adc: &mut Saadc) -> Result<(), ConfigErr> {
        use ConfigAction::*;

        match change {
            ResetPins => self.analog_enabled.clear(),
            DigitalPins(pin) => Err(ConfigErr::Unimplemented)?,
            AnalogPins(pin) => {
                if !ABILITIES.adc_pins.contains(&pin) {
                    return Err(ConfigErr::InvalidPin(pin));
                }
                // self.analog_pins[sampler as usize] = Some(AdcPin::from(pin));
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
    let analog_available = [AdcPin::P0_02(gpios.p0_02), AdcPin::P0_31(gpios.p0_31)];
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
