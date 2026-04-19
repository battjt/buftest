use anyhow::Result;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::env::args;
use std::io::{Write, stdin, stdout};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use clap::*;

#[derive(Parser, Clone)]
pub struct Cli {
    id: String,
    other_id: Option<String>,
}

// Log struct to log the sequence numbers and calculate the throughput once a second
struct Log {
    name: String,
    start: Instant,
    count: u64,
    last_log_time: Instant,
    last_log_sequence: u128,
}
impl Log {
    fn new(name: &str) -> Log {
        eprintln!("starting {name}");
        Log {
            name: name.to_string(),
            count: 0,
            last_log_sequence: 0,
            last_log_time: Instant::now(),
            // avoid division by zero in the first log
            start: Instant::now() - Duration::from_secs(1),
        }
    }
    // log once a second
    fn log(&mut self, sequence: u128) {
        // avoid calling Instant::now() too often, which is expensive, by only checking the sequence number
        if !sequence.is_multiple_of(10_000) {
            return;
        }
        let now = Instant::now();
        if (now - self.last_log_time) > Duration::from_secs(1) {
            let d = sequence - self.last_log_sequence;
            eprintln!(
                "{:6}: {}: {:12} {:12}/s",
                self.count,
                self.name,
                d,
                sequence / (now - self.start).as_secs() as u128
            );
            self.count += 1;
            self.last_log_time = now;
            self.last_log_sequence = sequence;
        }
    }
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(blue) = cli.other_id {
        let name = args().next().unwrap();
        let id = cli.id.clone();
        eprintln!("STARTING {name} {id} {blue}");
        let mut e1 = Command::new(name.as_str())
            .arg(id)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;
        let mut e2 = Command::new(name.as_str())
            .arg(blue)
            .stdout(Stdio::from(e1.stdin.take().unwrap()))
            .stdin(Stdio::from(e1.stdout.take().unwrap()))
            .spawn()?;

        e1.wait()?;
        e2.wait()?;
    } else {
        // tx
        let tx_name = cli.id.clone() + " tx";

        // flush stdout every 2ms to prevent buffering
        thread::spawn(|| -> Result<()> {
            let delay = Duration::from_millis(2);
            let stdout = stdout();
            loop {
                thread::sleep(delay);
                stdout.lock().flush()?;
            }
        });

        // thread to write sequence numbers to stdout
        thread::spawn(move || -> Result<()> {
            let mut log = Log::new(tx_name.as_str());
            let mut sequence = 0u128;
            let out = stdout();
            loop {
                out.lock().write_u128::<BE>(sequence)?;
                sequence += 1;
                log.log(sequence);
            }
        });

        // main thread reads sequence numbers from stdin
        let rx_name = cli.id.clone() + " rx";
        let mut log = Log::new(rx_name.as_str());
        let mut stdin = stdin();
        loop {
            let sequence = stdin.read_u128::<BE>()?;
            log.log(sequence);
        }
    }
    Ok(())
}
