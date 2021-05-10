#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::{buffered_uarte::BufferedUarte, interrupt, uarte};
use futures::pin_mut;

use rustyscope_traits::Message;
use nrf52832_hal::{
   pac::Peripherals,
   prelude::*,
   gpio::p0::Parts as P0Parts,
   saadc::{SaadcConfig, Saadc},
};

use defmt_setup::*;


#[embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nrf::Peripherals::take().unwrap();
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let mut tx_buffer = [0u8; 4096];
    let mut rx_buffer = [0u8; 4096];

    let irq = interrupt::take!(UARTE0_UART0);
    let u = unsafe {
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
    pin_mut!(u);

    let board = Peripherals::take().unwrap();
    let gpios = P0Parts::new(board.P0);

    // initialize saadc interface
    let saadc_config = SaadcConfig::default();
    let mut saadc = Saadc::new(board.SAADC, saadc_config);
    let mut saadc_pin = gpios.p0_02; // the pin your analog device is connected to

    // blocking read from saadc for `saadc_config.time` microseconds
    loop {
        let v = saadc.read(&mut saadc_pin).unwrap();
        info!("value: {}", v);
        Timer::after(Duration::from_secs(1)).await;
    }
}
