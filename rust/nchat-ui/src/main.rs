use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use nchat_core_sys::Core;
use nchat_ui::app::{App, TerminalGuard};
use nchat_ui::colorconf::Theme;
use nchat_ui::conf::{resolve_confdir, Conf};
use nchat_ui::protocol::{Event, ProfileInfo};
use nchat_ui::session::{describe, on_connect};
use nchat_ui::uiconf::UiConfig;

struct Args {
    confdir: String,
    headless: bool,
    seconds: u64,
}

fn parse_args() -> Result<Args, String> {
    let mut confdir = String::new();
    let mut headless = false;
    let mut seconds = 30;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-d" | "--confdir" => confdir = it.next().ok_or("--confdir needs a value")?,
            "--headless" => headless = true,
            "--seconds" => {
                seconds = it
                    .next()
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "--seconds needs a number")?
            }
            "-h" | "--help" => {
                println!("usage: nchat-rs [-d|--confdir DIR] [--headless] [--seconds N]");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(Args { confdir, headless, seconds })
}

fn load_conf_file(dir: &PathBuf, name: &str) -> Conf {
    let path = dir.join(name);
    std::fs::read_to_string(&path)
        .map(|s| Conf::parse(&s))
        .unwrap_or_default()
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let conf_path = resolve_confdir(&args.confdir);
    let ui_conf = UiConfig::from_conf(&load_conf_file(&conf_path, "ui.conf"));
    let color_conf = Theme::from_conf(&load_conf_file(&conf_path, "color.conf"));

    let (tx, rx) = mpsc::channel();
    let mut core = match Core::init(&args.confdir, tx) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cannot init core in {:?}: {e}", conf_path);
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

    if args.headless {
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
        return ExitCode::SUCCESS;
    }

    // TUI Mode
    let _guard = match TerminalGuard::init(ui_conf.mouse_enabled) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("failed to init terminal: {e}");
            return ExitCode::from(1);
        }
    };

    let mut app = match App::new(ui_conf, color_conf) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("failed to init app: {e}");
            return ExitCode::from(1);
        }
    };
    app.model.set_profiles(profiles.clone());

    let exit_flag = Arc::new(AtomicBool::new(false));
    let ef = Arc::clone(&exit_flag);

    let _ = app.run(rx, ef, &mut core);

    // Watchdog stop: don't hang forever if core.stop() wedges.
    let stop_thread = std::thread::spawn(move || {
        core.stop();
    });
    let timeout = Duration::from_secs(3);
    let start = Instant::now();
    while !stop_thread.is_finished() {
        if start.elapsed() >= timeout {
            eprintln!("core stop timed out after 3s; exiting");
            return ExitCode::SUCCESS;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if stop_thread.join().is_err() {
        eprintln!("core stop joined with error");
    }

    ExitCode::SUCCESS
}
