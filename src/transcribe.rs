use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use arboard::Clipboard;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

use crate::audio::{rms, SAMPLE_RATE};
use crate::paste::paste_via_clipboard;

pub struct TranscribeConfig {
    pub model_path: PathBuf,
    pub unload_timeout_mins: u64,
    pub language: String,
    pub initial_prompt: String,
    pub beam_size: i32,
    pub suppress_blank: bool,
    pub no_speech_threshold: f32,
    pub min_samples: usize,
    pub min_rms: f32,
}

pub fn spawn_worker(rx: Receiver<Vec<f32>>, cfg: TranscribeConfig) {
    std::thread::spawn(move || {
        let mut clipboard = Clipboard::new().expect("failed to init clipboard");
        // (WhisperContext, WhisperState) — loaded lazily, dropped on idle timeout
        let mut loaded: Option<(WhisperContext, WhisperState)> = None;

        loop {
            let recv = if cfg.unload_timeout_mins > 0 {
                rx.recv_timeout(Duration::from_secs(cfg.unload_timeout_mins * 60))
            } else {
                rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
            };

            match recv {
                Ok(audio) => {
                    if audio.len() < cfg.min_samples || rms(&audio) < cfg.min_rms {
                        continue;
                    }

                    if loaded.is_none() {
                        println!(
                            "Loading model from {}...",
                            cfg.model_path.display()
                        );
                        let ctx = WhisperContext::new_with_params(
                            cfg.model_path.to_str().expect("model path not valid UTF-8"),
                            WhisperContextParameters::default(),
                        )
                        .expect("failed to load model");
                        let state = ctx.create_state().expect("failed to create whisper state");
                        loaded = Some((ctx, state));
                        println!("Model loaded.");
                    }

                    println!(
                        "Transcribing {} samples ({:.1}s)...",
                        audio.len(),
                        audio.len() as f32 / SAMPLE_RATE as f32
                    );

                    let (_, state) = loaded.as_mut().unwrap();
                    let strategy = if cfg.beam_size > 1 {
                        SamplingStrategy::BeamSearch {
                            beam_size: cfg.beam_size,
                            patience: -1.0,
                        }
                    } else {
                        SamplingStrategy::Greedy { best_of: 1 }
                    };
                    let mut params = FullParams::new(strategy);
                    params.set_language(Some(&cfg.language));
                    params.set_initial_prompt(&cfg.initial_prompt);
                    params.set_suppress_blank(cfg.suppress_blank);
                    params.set_no_speech_thold(cfg.no_speech_threshold);
                    params.set_print_progress(false);
                    params.set_print_realtime(false);
                    params.set_print_timestamps(false);

                    state.full(params, &audio).expect("whisper inference failed");

                    let n = state.full_n_segments();
                    let mut raw = String::new();
                    for i in 0..n {
                        if let Some(seg) = state.get_segment(i) {
                            if let Ok(s) = seg.to_str_lossy() {
                                raw.push_str(&s);
                            }
                        }
                    }
                    let text = raw.trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    println!("→ {text}");
                    paste_via_clipboard(&mut clipboard, &text);
                }

                Err(RecvTimeoutError::Timeout) => {
                    if loaded.is_some() {
                        loaded = None;
                        println!(
                            "Model unloaded after {} min inactivity.",
                            cfg.unload_timeout_mins
                        );
                    }
                }

                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}
