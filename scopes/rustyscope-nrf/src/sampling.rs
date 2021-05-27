use crate::hal::saadc::Saadc;
use crate::hal::pac::SAADC;
use embedded_hal::adc::OneShot;
use embassy::time::{Timer, Duration};
use rustyscope_traits::SampleKind;
use itertools::izip;
use crate::Mode;
use crate::Config;
use crate::Mutex;
use crate::Serial;

use core::ops::{Deref, DerefMut};

use crate::AdcPin;
use futures_intrusive::channel::LocalChannel;
pub type Channel = LocalChannel<i16, [i16; 32]>;

fn sample(adc: &mut Saadc, pin: &mut AdcPin) -> i16 {
    match pin {
        AdcPin::P0_31(p) => adc.read(p).unwrap(),
        AdcPin::P0_02(p) => adc.read(p).unwrap(),
    }
}

pub async fn sample_loop<'a, 'd>(serial: &Serial<'a, 'd>, mode: &Mutex<Mode>, config: &Config, channel: &Channel, saadc: SAADC) {
    let saadc_config = crate::hal::saadc::SaadcConfig::default();
    let mut adc = Saadc::new(saadc, saadc_config);

    loop {
        use SampleKind::*;
        let curr_mode = {
            let guard = mode.lock().await;
            let m = *&guard.deref();
            *m
        };

        match curr_mode {
            Mode::Idle => Timer::after(Duration::from_millis(500)).await,
            Mode::Continues(Analog) => {
                todo!("not yet finished, see TODO");
                let mut guard = config.0.lock().await;
                let config = guard.deref_mut();
                for pin in &mut config.analog_enabled {
                    let val = sample(&mut adc, pin);
                    channel.send(val).await.unwrap();
                }
                Timer::after(Duration::from_millis(500)).await;
            }
            Mode::Continues(Digital) => todo!(),
            Mode::Burst(Analog) => {
                let mut data = [0i16; 2_000];
                let mut guard = config.0.lock().await;
                let config = guard.deref_mut();

                let len = config.analog_enabled.len();
                let pins = &mut config.analog_enabled;
                let values = data.iter_mut();
                for (i, val) in values.enumerate() {
                    let pin = &mut pins[i % len];
                    *val = sample(&mut adc, pin);
                }
                serial.send_data(&data).await;
                let mut new_mode = mode.lock().await;
                let new_mode = new_mode.deref_mut();
                *new_mode = Mode::Idle;
            }
            Mode::Burst(Digital) => todo!(),
            Mode::Err(ref e) => {
                defmt::error!("config err occured: {}", e);
                let mut new_mode = mode.lock().await;
                let new_mode = new_mode.deref_mut();
                *new_mode = Mode::Idle;
            },
        }
    }
}
