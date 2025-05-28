use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};
use notify_rust::Notification;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Focus,
    Rest,
}

impl Mode {
    /// Return the human-readable name of this mode.
    fn as_str(&self) -> &'static str {
        match self {
            Mode::Focus => "focus",
            Mode::Rest  => "rest",
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "tomatl-cli", about = "Manage focus and rest sessions")]
struct Cli {
    mode: Mode,
    minutes: f32
}

fn main() {
    let args = Cli::parse();
    let mode_name = args.mode.as_str();
    let minutes = args.minutes;
    println!("Starting a {} session for {} minutes", mode_name, minutes);

    let total_secs = (minutes * 60.0) as i32;
    let pb = ProgressBar::new(total_secs as u64);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} sec")
            .unwrap()
            .progress_chars("##-"),
    );

    for _ in 0..total_secs {
        pb.inc(1);
        thread::sleep(Duration::from_secs(1));
    }

    Notification::new()
        .summary("timer up!")
        .body(&format!("Your {mode_name} session is complete."))
        .show()
        .unwrap();
}



