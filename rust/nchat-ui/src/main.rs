use std::process::ExitCode;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use nchat_core_sys::Core;
use nchat_ui::protocol::{Event, ProfileInfo};
use nchat_ui::session::{describe, on_connect};

struct Args {
    confdir: String,
    seconds: u64,
}

fn parse_args() -> Result<Args, String> {
    let mut confdir = String::new();
    let mut seconds = 30;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-d" | "--confdir" => confdir = it.next().ok_or("--confdir needs a value")?,
            "--seconds" => {
                seconds = it
                    .next()
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "--seconds needs a number")?
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(Args { confdir, seconds })
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let (tx, rx) = mpsc::channel();
    let mut core = match Core::init(&args.confdir, tx) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cannot init core in {}: {e} (is nchat already running on this confdir?)", args.confdir);
            return ExitCode::from(1);
        }
    };

    let profiles: Vec<ProfileInfo> = match core.start().map(|j| serde_json::from_str(&j)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            eprintln!("bad profile list: {e}");
            return ExitCode::from(1);
        }
        Err(e) => {
            eprintln!("cannot start core: {e}");
            return ExitCode::from(1);
        }
    };
    println!("profiles: {}", profiles.iter().map(|p| p.id.as_str()).collect::<Vec<_>>().join(", "));

    let deadline = Instant::now() + Duration::from_secs(args.seconds);
    while Instant::now() < deadline {
        let raw = match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(raw) => raw,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let event: Event = match serde_json::from_str(&raw) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("unparsable event ({e}): {raw}");
                continue;
            }
        };
        if let Event::Connect { profile, success: true } = &event {
            if let Some(info) = profiles.iter().find(|p| &p.id == profile) {
                for request in on_connect(info) {
                    if let Err(e) = core.send(&request.to_json()) {
                        eprintln!("request failed: {e}");
                    }
                }
            }
        }
        println!("{}", describe(&event));
    }

    core.stop();
    ExitCode::SUCCESS
}
