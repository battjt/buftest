use anyhow::Result;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::env::args;
use std::io::{Write, stdin, stdout};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use clap::*;

#[derive(Parser, Clone)]
pub struct MainArgs {
    red: Option<String>,
    blue: Option<String>,
}

struct Log {
    name: String,
    count: u64,
    last: Instant,
    start: Instant,
    sequence: u64,
}
impl Log {
    fn new(name: &str) -> Log {
        eprintln!("starting {name}");
        Log {
            name: name.to_string(),
            count: 0,
            sequence: 0,
            last: Instant::now(),
            start: Instant::now(),
        }
    }
    fn log(&mut self, sequence: u64) {
        let now = Instant::now();
        if (now - self.last) > Duration::from_secs(1) {
            let d = sequence - self.sequence;
            eprintln!(
                "{:6}: {}: {:12} {:12}/s",
                self.count,
                self.name,
                d,
                (sequence as u128 * 1000u128) / (now - self.start).as_millis()
            );
            self.count += 1;
            self.last = now;
            self.sequence = sequence;
        }
    }
}
fn main() -> Result<()> {
    let mainargs = MainArgs::parse();
    if let (Some(red), Some(blue)) = (mainargs.red.clone(), mainargs.blue) {
        let name = args().next().unwrap();
        eprintln!("STARTING {name} {red} {blue}");
        let mut e1 = Command::new(name.as_str())
            .arg(red)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;
        let mut e2 = Command::new(name.as_str())
            .arg(blue)
            .stdout(Stdio::from(e1.stdin.unwrap()))
            .stdin(Stdio::from(e1.stdout.unwrap()))
            .spawn()?;

        //e1.wait();
        e2.wait()?;
    } else if let Some(name) = mainargs.red {
        // send
        let tx_name = name.clone() + " tx";
        thread::spawn(|| -> Result<()> {
            let delay = Duration::from_millis(8);
            let stdout = stdout();
            loop {
                thread::sleep(delay);
                stdout.lock().flush()?;
            }
        });

        thread::spawn(move || -> Result<()> {
            let mut log = Log::new(tx_name.as_str());
            let mut sequence = 0u128;
            let out = stdout();
            loop {
                out.lock().write_u128::<BE>(sequence)?;
                sequence += 1;
                log.log(sequence as u64);
            }
        });
        //receive
        let rx_name = name.clone() + " rx";
        let mut log = Log::new(rx_name.as_str());
        let mut stdin = stdin();
        loop {
            let sequence = stdin.read_u128::<BE>()? as u64;
            log.log(sequence);
        }
    }
    Ok(())
}
