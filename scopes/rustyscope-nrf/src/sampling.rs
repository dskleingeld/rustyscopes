use crate::hal::saadc::Saadc;
use crate::hal::pac::SAADC;
use embedded_hal::adc::OneShot;
use embassy::time::{Timer, Duration, Instant};
use rustyscope_traits::SampleKind;
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
        AdcPin::P0_02(p) => adc.read(p).unwrap(),
        AdcPin::P0_03(p) => adc.read(p).unwrap(),
        AdcPin::P0_04(p) => adc.read(p).unwrap(),
        AdcPin::P0_05(p) => adc.read(p).unwrap(),
        AdcPin::P0_28(p) => adc.read(p).unwrap(),
        AdcPin::P0_29(p) => adc.read(p).unwrap(),
        AdcPin::P0_30(p) => adc.read(p).unwrap(),
        AdcPin::P0_31(p) => adc.read(p).unwrap(),
    }
}

pub async fn sample_loop<'a, 'd>(serial: &Serial<'a, 'd>, mode: &Mutex<Mode>, config: &Config, channel: &Channel, saadc: SAADC) {
    use crate::hal::saadc::{SaadcConfig, Reference, Gain};
    let mut saadc_config = SaadcConfig { 
        reference: Reference::VDD1_4,
        gain: Gain::GAIN1_4,
        ..SaadcConfig::default() };
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
                let start = Instant::now();
                let mut next = start;
                for (i, val) in values.enumerate() {
                    let idx = i % len;
                    let pin = &mut pins[idx];
                    busy_wait_till(next);
                    *val = sample(&mut adc, pin);
                    next += config.sample_period.unwrap_or(Duration::from_secs(0));
                }
                let duration = start.elapsed().as_micros();
                serial.send_burst_data(&data, duration).await;

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

fn busy_wait_till(time: Instant) {
    while Instant::now() < time { continue }
}
