use std::io::Write;
use std::time::Duration;
use std::thread;

use rustyscope_traits::{Command, Reply, SampleKind, ConfigAction};
use ferrous_serialport as serialport;
use ferrous_serialport::SerialPort;
use std::path::PathBuf;
use std::convert::TryFrom;

#[derive(structopt::StructOpt, Debug)]
#[structopt(name = "scope viewer")]
struct Args {
    /// path to the serial port
    #[structopt(short, long)]
    port: PathBuf,
}

fn print_incoming(mut serial: Box<dyn SerialPort>) {
    use std::io::ErrorKind::TimedOut;
    let mut buf = [0u8; Command::SIZE];
    loop {
        dbg!();
        match serial.read_exact(&mut buf) {
            Err(e) if e.kind() == TimedOut => continue, 
            Err(e) => panic!("{}", e),
            Ok(()) => (),
        }
        dbg!();
        
        let len = match Reply::try_from(&buf).unwrap() {
            Reply::Ok => continue,
            Reply::Err(config_err) => panic!("config err: {:?}", config_err),
            Reply::Data(len) => len,
        };

        println!("reading {} bytes of data", len);
        let mut data = vec![0u8; len as usize];
        serial.read_exact(&mut data).unwrap();
        println!("got data len: {}", data.len());
    }
}


#[paw::main]
fn main(args: Args) -> Result<(), std::io::Error> {
    let mut serial = serialport::new(args.port.to_str().unwrap(), 9600)
        .parity(serialport::Parity::None)
        .flow_control(serialport::FlowControl::Hardware)
        .timeout(Duration::from_secs(20))
        .open()
        .unwrap();

    let read_port = serial.try_clone().unwrap();
    let handle = thread::spawn(move || print_incoming(read_port));
    
    let cmd = Command::Config(ConfigAction::ResetPins);
    serial.write_all(&cmd.serialize()).unwrap();

    let cmd = Command::Config(ConfigAction::AnalogPins(2));
    serial.write_all(&cmd.serialize()).unwrap();
    
    let cmd = Command::Continues(SampleKind::Analog);
    serial.write_all(&cmd.serialize()).unwrap();
    

    handle.join().unwrap();
    println!("all done");

    Ok(())
}
