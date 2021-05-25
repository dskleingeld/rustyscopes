#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(const_fn_trait_bound)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![feature(array_methods)]
#![allow(incomplete_features)]

mod defmt_setup;

#[allow(unused_imports)]
use defmt::panic; // needed for embassy main
use embassy_nrf::Peripherals;
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

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    #[allow(non_snake_case)]
    let embassy_nrf::Peripherals{UARTE0, TIMER0, PPI_CH0, PPI_CH1, P0_08, P0_06, P0_05, P0_07, ..} = p;

    let mut tx_buffer = [0u8; 4096];
    let mut rx_buffer = [0u8; 265];
    let uart = Serial::setup_uart(UARTE0, P0_08, P0_06, P0_05, P0_07);
    pin_mut!(uart);
    let serial = Serial::from_pinned_uart(uart);
    let config = config::Config::init();

    let mode = Mutex::new(Mode::Idle, false);

    let sample = sampling::samle_loop(&mode, &config);
    let send_data = communications::send_data(&serial);
    let handle_commands = communications::handle_commands(&serial, &mode, &config);

    handle_commands.await;
    // futures::join!(handle_commands, send_data, sample);
    defmt::error!("should never get here");
}
