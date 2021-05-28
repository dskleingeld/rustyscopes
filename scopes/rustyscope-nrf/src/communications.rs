use embassy_nrf::peripherals::{P0_05, P0_06, P0_07, P0_08};
use embassy_nrf::peripherals::UARTE0;
use embassy_nrf::{uarte, interrupt};
use embassy_nrf::uarte::Uarte;
use embassy::traits::uart::{Read, Write};
use rustyscope_traits::{Command, Reply};
use core::pin::Pin;
use core::ops::DerefMut;
use core::convert::TryFrom;

use crate::Mode;
use crate::mutex::Mutex;
use crate::config::Config;
use crate::sampling::Channel;

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

    // TODO this blocks until a new command is recieved
    // which keeps the mutex locked blocking sending
    // need a serial that has seperate objects for 
    // reading and writing
    pub async fn read_command(&self) -> Command {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();
        let mut buf = [0u8; Command::SIZE];
        serial.read(&mut buf).await.unwrap();
        Command::try_from(&buf).unwrap()
    }

    pub async fn send_reply(&self, reply: Reply) {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();
        let buf = reply.serialize();
        serial.write(&buf).await.unwrap();
    }

    pub async fn send_data(&self, data: [i16;8]) {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();

        let buffer = bytemuck::cast_slice(&data);
        let data = Reply::Data(data.len() as u32).serialize();
        serial.write(&data).await.unwrap();
        serial.write(&buffer).await.unwrap();
    }

    pub async fn send_burst_data(&self, data: &[i16], duration: u64) {
        let mut m = self.0.lock().await;
        let serial = m.deref_mut();

        let data = bytemuck::cast_slice(data);
        for chunk in data.chunks(u8::MAX.into()) {
            let data = Reply::Data(chunk.len() as u32).serialize();
            serial.write(&data).await.unwrap();
            serial.write(&chunk).await.unwrap();
        }
        let done = Reply::Done(duration as u32).serialize();
        serial.write(&done).await.unwrap();
    }
}

pub async fn handle_commands<'a, 'd>(serial: &Serial<'a, 'd>, mode: &Mutex<Mode>, config: &Config) {
    loop {
        let command = serial.read_command().await;
        defmt::info!("got command: {}", command);

        let new_mode = match command {
            Command::Stop => Some(Mode::Idle),
            Command::Continues(s) => Some(Mode::Continues(s)),
            Command::Burst(s) => Some(Mode::Burst(s)),
            Command::Config(change) => match config.apply(change).await {
                Result::Ok(_) => None, //*new_mode,
                Result::Err(e) => {
                    let reply = Reply::Err(e);
                    serial.send_reply(reply).await;
                    Some(Mode::Err(e))
                }
            },
        };

        if let Some(new) = new_mode {
            let mut m = mode.lock().await;
            let mode = m.deref_mut();
            *mode = new;
        }
    }
}

pub async fn send_data<'d,'a>(serial: &Serial<'d,'a>, channel: &Channel) {
    let mut data = [0i16;8];
    loop {
        for i in &mut data {
            *i = channel.receive().await.unwrap();
        }
        serial.send_data(data).await;
    }
}
