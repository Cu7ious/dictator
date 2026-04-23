mod audio;
mod config;
mod hotkey;
mod paste;
mod sounds;
mod transcribe;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use clap::Parser;

#[derive(Parser)]
#[command(name = "dictator", about = "Push-to-talk dictation for macOS")]
struct Cli {
    /// Print config file locations, which is active, and whether each exists
    #[arg(long)]
    config_info: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.config_info {
        print_config_info();
        return;
    }

    let cfg = config::load();

    println!("Ready. Hold {} to dictate.", cfg.general.hotkey);

    let recording = Arc::new(AtomicBool::new(false));
    let audio_buf: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();
    let (icon_tx, icon_rx) = mpsc::channel::<bool>();

    let icon_idle = tray::load_icon("assets/tray-idle.png");
    let icon_active = tray::load_icon("assets/tray-recording.png");
    let tray = tray::build(icon_idle.clone());

    let min_samples = (cfg.audio.min_duration_secs * audio::SAMPLE_RATE as f32).round() as usize;

    audio::spawn_capture(Arc::clone(&recording), Arc::clone(&audio_buf));

    transcribe::spawn_worker(
        audio_rx,
        transcribe::TranscribeConfig {
            model_path: config::expand_path(&cfg.model.path),
            unload_timeout_mins: cfg.model.unload_timeout_mins,
            language: cfg.whisper.language,
            initial_prompt: cfg.whisper.initial_prompt,
            beam_size: cfg.whisper.beam_size,
            suppress_blank: cfg.whisper.suppress_blank,
            no_speech_threshold: cfg.whisper.no_speech_threshold,
            min_samples,
            min_rms: cfg.audio.silence_threshold,
        },
    );

    hotkey::spawn_listener(
        Arc::clone(&recording),
        Arc::clone(&audio_buf),
        audio_tx,
        icon_tx,
        config::parse_key(&cfg.general.hotkey),
        cfg.sounds.start,
        cfg.sounds.stop,
        cfg.sounds.enabled,
        cfg.sounds.record_delay_ms,
    );

    loop {
        tray::run_loop_tick(0.05);
        tray::process_icon_updates(&tray, &icon_rx, &icon_idle, &icon_active);
    }
}

fn print_config_info() {
    let locations = config::config_locations();
    let mut active: Option<&str> = None;

    println!("Config locations (checked in order):");
    for (label, path) in &locations {
        let exists = path.exists();
        let marker = if exists { "✓" } else { "✗" };
        println!("  [{marker}] {label}");
        println!("      {}", path.display());
        if exists && active.is_none() {
            active = Some(label);
        }
    }

    println!();
    match active {
        Some(label) => println!("Active: {label}"),
        None => println!("Active: none — defaults will be written to ~/.config/dictator/config.toml on first run"),
    }
}
