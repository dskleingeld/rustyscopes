use crate::hal::gpio;
use crate::hal::saadc::Saadc;
use embedded_hal::adc::{self, OneShot};
use embassy::time::{Timer, Duration};
use rustyscope_traits::SampleKind;
use crate::defmt_setup::*;
use crate::Mode;
use crate::config::Config;
use crate::mutex::Mutex;

use core::ops::{Deref, DerefMut};

/* let saadc_config = SaadcConfig::default();
let mut adc = Saadc::new(board.SAADC, saadc_config); */

enum AdcPin {
    P0_31(gpio::p0::P0_31<gpio::Disconnected>),
    P0_02(gpio::p0::P0_02<gpio::Disconnected>),
}

struct AdcPins {
    p0_31: Option<gpio::p0::P0_31<gpio::Disconnected>>,
    p0_02: Option<gpio::p0::P0_02<gpio::Disconnected>>,
}

async fn sample<PIN>(adc: &mut Saadc, pin: &mut PIN)
where
    PIN: adc::Channel<Saadc, ID = u8>,
{
    let v = adc.read(pin).unwrap();
    info!("value: {}", v);
    Timer::after(Duration::from_secs(1)).await;
}

pub async fn samle_loop(mode: &Mutex<Mode>, config: &Config) {
    loop {
        use SampleKind::*;

        let m = mode.lock().await;
        match &m.deref() {
            Mode::Idle => (),
            Mode::Continues(Analog) => (),
            Mode::Continues(Digital) => (),
            Mode::Burst(Analog) => (),
            Mode::Burst(Digital) => (),
            Mode::Err(ref e) => {
                defmt::error!("config err occured: {}", e);
                let mut new_mode = mode.lock().await;
                let new_mode = new_mode.deref_mut();
                *new_mode = Mode::Idle;
            },
        }
    }
}
