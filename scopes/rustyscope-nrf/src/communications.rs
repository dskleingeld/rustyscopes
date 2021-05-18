use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::peripherals::{P0_06, P0_08};
use embassy_nrf::peripherals::{PPI_CH0, PPI_CH1, TIMER0, UARTE0};
use embassy_nrf::uarte;
use embassy_nrf::{buffered_uarte::BufferedUarte, interrupt};
use rustyscope_traits::Command;
use core::pin::Pin;
use futures::pin_mut;
use core::ops::{Deref, DerefMut};

use crate::Mode;
use crate::mutex::Mutex;
use crate::config::Config;

pub struct Serial<'a,'d>(pub Mutex<Pin<&'a mut BufferedUarte<'d, UARTE0, TIMER0>>>);


impl<'a,'d> Serial<'a,'d> {
    pub fn setup_uart(
        uart: UARTE0,
        timer: TIMER0,
        ppi_a: PPI_CH0,
        ppi_b: PPI_CH1,
        rxd: P0_08,
        txd: P0_06,
        tx_buffer: &'d mut [u8; 4096],
        rx_buffer: &'d mut [u8; 265],
    ) -> BufferedUarte<'d, UARTE0, TIMER0> {
        let mut config = uarte::Config::default();
        config.parity = uarte::Parity::EXCLUDED;
        config.baudrate = uarte::Baudrate::BAUD115200;

        let irq = interrupt::take!(UARTE0_UART0);
        unsafe {
            BufferedUarte::new(
                uart, timer, ppi_a, ppi_b, irq, rxd, txd, NoPin, NoPin, config, rx_buffer, tx_buffer,
            )
        }
    }
    pub fn from_pinned_uart(uart: Pin<&'a mut BufferedUarte<'d, UARTE0, TIMER0>>) -> Self {
        Self(Mutex::new(uart, true))
    }

    pub async fn read_command(&self) -> Command {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();
        let mut buf = [0u8; 8];
        serial.read_exact(&mut buf);
        // Command::from(buf)
        Command::Stop
    }
}
/*
pub async fn handle_commands<'d>(serial: Pin<&Serial<'d>>, mode: &Mutex<Mode>, config: &Config) {
    loop {
        // serial.
        let command = Command::Stop;

        let mut m = mode.lock().await;
        let new_mode = m.deref_mut();
        *new_mode = match command {
            Command::Stop => Mode::Idle,
            Command::Continues(s) => Mode::Continues(s),
            Command::Burst(s) => Mode::Burst(s),
            Command::Config(change) => match config.apply(change).await {
                Result::Ok(_) => *mode.lock().await.deref(),
                Result::Err(e) => Mode::Err(e),
            },
        };
    }
}

pub async fn send_data<'d>(serial: Pin<&Serial<'d>>) {} */
