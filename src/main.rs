use clap::Parser;
use colored::Colorize;
use figlet_rs::FIGfont;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use notify_rust::Notification;
use rodio::{Decoder, OutputStream, Sink};
use std::{thread, time::Duration};

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
            Mode::Rest => "rest",
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "tomatl-cli", about = "Manage focus and rest sessions")]
struct Cli {
    mode: Mode,
    minutes: f32,
}

fn main() {
    let args = Cli::parse();
    let mode = &args.mode.as_str();
    let minutes = args.minutes;

    // 1) ASCII-art header
    let font = FIGfont::standard().unwrap();
    let figure = font.convert(mode).unwrap();
    println!("\n{}\n", figure.to_string().cyan().bold());

    // 2) Subheader with emoji
    println!(
        "{}\n",
        format!(
            "Starting a {} session for {} minutes ⏱️",
            mode.green(),
            minutes
        )
        .magenta()
        .bold()
    );

    // 3) Spinner + progress bar
    let total_secs = (minutes * 60.0) as u64;
    let mp = MultiProgress::new();

    let spinner = mp.add(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message("Good luck!");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let pb = mp.add(ProgressBar::new(total_secs));
    pb.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>3}/{len:3} sec • ETA {eta_precise}")
            .unwrap()
            .progress_chars("█▇▆▅▄▃▂▁ "),
    );

    // 4) Run!
    for _ in 0..total_secs {
        pb.inc(1);
        thread::sleep(Duration::from_secs(1));
    }
    spinner.finish_and_clear();
    pb.finish_with_message("🎉 Done!");

    // 5) Desktop notification
    Notification::new()
        .summary("Timer up!")
        .body(&format!("Your {} session is complete.", mode))
        .show()
        .unwrap();

    // 6) Play sound
    if let Err(e) = play_sound() {
        eprintln!("Error playing sound: {}", e);
    }
}

fn play_sound() -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;

    let sink = Sink::try_new(&stream_handle)?;

    static SOUND: &[u8] = include_bytes!("../assets/sound.mp3");
    let cursor = std::io::Cursor::new(SOUND);
    let source = Decoder::new(cursor)?;

    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
