#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(const_fn_trait_bound)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;

#[allow(unused_imports)]
use defmt::panic; // needed for embassy main
use embassy::executor::Spawner;
// use embassy::time::{Duration, Timer};
use core::marker::PhantomPinned;
use futures::pin_mut;

mod description;
mod communications;
mod config;
mod sampling;
mod mutex;
use nrf52832_hal as hal;

use mutex::Mutex;
use communications::Serial;
use rustyscope_traits::{SampleKind, ConfigErr};

#[allow(unused_imports)]
use defmt_setup::*;

#[derive(Copy, Clone)]
pub enum Mode {
    Idle,
    Continues(SampleKind),
    Burst(SampleKind),
    Err(ConfigErr),
}

use embassy_nrf::peripherals::{UARTE0, TIMER0};
use embassy_nrf::buffered_uarte::BufferedUarte;
use core::pin::Pin;
async fn test<'d>(serial: &Mutex<Pin<&mut BufferedUarte<'d, UARTE0, TIMER0>>>) {
    let mut s = serial.lock().await;

    let mut buf = [0u8; 8];
    use embassy::io::AsyncBufReadExt;
    s.read_exact(&mut buf);
}

pub struct Test<'d, 'a>(pub Mutex<core::pin::Pin<&'d mut BufferedUarte<'a, embassy_nrf::peripherals::UARTE0, embassy_nrf::peripherals::TIMER0>>>);

#[embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nrf::Peripherals::take().unwrap();
    #[allow(non_snake_case)]
    let embassy_nrf::Peripherals{UARTE0, TIMER0, PPI_CH0, PPI_CH1, P0_08, P0_06, ..} = p;

    let mut tx_buffer = [0u8; 4096];
    let mut rx_buffer = [0u8; 265];
    let uart = Serial::setup_uart(UARTE0, TIMER0, PPI_CH0, PPI_CH1, P0_08, P0_06, &mut tx_buffer, &mut rx_buffer);
    pin_mut!(uart);
    let serial = Serial::from_pinned_uart(uart);
    let config = config::Config::init();

    let mode = Mutex::new(Mode::Idle, false);

    serial.read_command();

    let sample = sampling::samle_loop(&mode, &config);
    // let send_data = communications::send_data(serial);
    // let handle_commands = communications::handle_commands(serial, &mode, &config);

    // futures::join!(handle_commands, send_data, sample);
}
