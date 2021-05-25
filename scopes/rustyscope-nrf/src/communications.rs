use embassy_nrf::gpio::NoPin;
use embassy_nrf::peripherals::{P0_05, P0_06, P0_07, P0_08};
use embassy_nrf::peripherals::{PPI_CH0, PPI_CH1, TIMER0, UARTE0};
use embassy_nrf::{uarte, interrupt};
use embassy_nrf::uarte::Uarte;
use embassy::traits::uart::{Read, Write};
use embassy_nrf::buffered_uarte::BufferedUarte;
use rustyscope_traits::{Command, Reply};
use core::pin::Pin;
use futures::pin_mut;
use core::ops::{Deref, DerefMut};
use core::convert::TryFrom;
use defmt::info;

use crate::Mode;
use crate::mutex::Mutex;
use crate::config::Config;

pub struct Serial<'a,'d>(pub Mutex<Pin<&'a mut Uarte<'d, UARTE0>>>);


impl<'a,'d> Serial<'a,'d> {
    pub fn setup_uart(
        uart: UARTE0,
        rxd: P0_08,
        txd: P0_06,
        cts: P0_05,
        rts: P0_07,
    ) -> Uarte<'d, UARTE0> {
        let mut config = uarte::Config::default();
        config.parity = uarte::Parity::EXCLUDED;
        config.baudrate = uarte::Baudrate::BAUD9600;

        let irq = interrupt::take!(UARTE0_UART0);
        unsafe {
            Uarte::new( // note rts is connected to cts
                uart, irq, rxd, txd, rts, cts, config,
            )
        }
    }

    pub fn from_pinned_uart(uart: Pin<&'a mut Uarte<'d, UARTE0>>) -> Self {
        Self(Mutex::new(uart, true))
    }

    pub async fn read_command(&self) -> Command {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();
        let mut buf = [0u8; Command::SIZE];
        serial.read(&mut buf).await;
        Command::try_from(&buf).unwrap()
    }

    pub async fn send_reply(&self, reply: Reply) {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();
        let buf = reply.serialize();
        serial.write(&buf).await;
    }
}

pub async fn handle_commands<'a, 'd>(serial: &Serial<'a, 'd>, mode: &Mutex<Mode>, config: &Config) {
    loop {
        // serial.
        let command = serial.read_command().await;
        defmt::debug!("command is: {}", command);

        let mut m = mode.lock().await;
        let new_mode = m.deref_mut();
        *new_mode = match command {
            Command::Stop => Mode::Idle,
            Command::Continues(s) => Mode::Continues(s),
            Command::Burst(s) => Mode::Burst(s),
            Command::Config(change) => match config.apply(change).await {
                Result::Ok(_) => *new_mode,
                Result::Err(e) => {
                    let reply = Reply::Err(e);
                    serial.send_reply(reply).await;
                    Mode::Err(e)
                }
            },
        };
    }
}

pub async fn send_data<'d,'a>(serial: &Serial<'d,'a>) {}
