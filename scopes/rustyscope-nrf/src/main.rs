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
use futures::pin_mut;

mod description;
mod communications;
mod config;
mod sampling;
mod mutex;
use nrf52832_hal as hal;
use crate::hal::pac;

use mutex::Mutex;
use config::{Config, AdcPin};
use sampling::Channel;
use communications::Serial;
use rustyscope_traits::{SampleKind, ConfigErr};

#[allow(unused_imports)]
use defmt_setup::*;

#[derive(Copy, Clone, defmt::Format)]
pub enum Mode {
    Idle,
    Continues(SampleKind),
    Burst(SampleKind),
    Err(ConfigErr),
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    #[allow(non_snake_case)]
    let Peripherals{UARTE0, P0_08, P0_06, P0_05, P0_07, ..} = p;
    let uart = Serial::setup_uart(UARTE0, P0_08, P0_06, P0_05, P0_07);
    pin_mut!(uart);

    let b = pac::Peripherals::take().unwrap();
    #[allow(non_snake_case)]
    let pac::Peripherals{SAADC, P0,..} = b;

    let serial = Serial::from_pinned_uart(uart);
    let config = Config::from_gpios(P0);
    let mode = Mutex::new(Mode::Idle, false);
    let channel = Channel::new();

    let sample = sampling::sample_loop(&serial, &mode, &config, &channel, SAADC);
    let send_data = communications::send_data(&serial, &channel);
    let handle_commands = communications::handle_commands(&serial, &mode, &config);

    futures::join!(handle_commands, send_data, sample);
    defmt::error!("should never get here");
}
