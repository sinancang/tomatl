use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
use figlet_rs::FIGfont;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use notify_rust::Notification;
use rodio::{Decoder, OutputStream, Sink};
use rusqlite::{Connection, Result, params};
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

/// Initializes (or migrates) the database: creates `sessions` table if it doesn't exist.
fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            start_iso      TEXT NOT NULL,
            minutes        FLOAT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

/// Inserts a new focus session into `sessions`.
/// 
/// - `start` is the UTC timestamp when the session began.
/// - `minutes` is the length of that session in.
fn record_session(conn: &Connection, start: DateTime<Utc>, minutes: f32) -> Result<()> {
    let start_iso = start.to_rfc3339(); // e.g. "2025-05-30T14:23:00+00:00"
    conn.execute(
        "INSERT INTO sessions (start_iso, minutes) VALUES (?1, ?2)",
        params![start_iso, minutes],
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("focus.db")?;
    init_db(&conn)?;
    
    let args = Cli::parse();
    let mode = &args.mode.as_str();
    let minutes = args.minutes;
    let now = Utc::now();

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
    record_session(&conn, now, minutes)?;
    Ok(())
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
