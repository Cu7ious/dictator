use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Whisper requires exactly 16 kHz — not user-configurable.
pub const SAMPLE_RATE: u32 = 16_000;

pub fn rms(audio: &[f32]) -> f32 {
    let sum: f32 = audio.iter().map(|s| s * s).sum();
    (sum / audio.len() as f32).sqrt()
}

pub fn resample(data: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to {
        return data.to_vec();
    }
    let ratio = from as f64 / to as f64;
    let out_len = (data.len() as f64 / ratio) as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = pos as usize;
            let frac = (pos - idx as f64) as f32;
            let a = data.get(idx).copied().unwrap_or(0.0);
            let b = data.get(idx + 1).copied().unwrap_or(0.0);
            a + (b - a) * frac
        })
        .collect()
}

pub fn spawn_capture(recording: Arc<AtomicBool>, audio_buf: Arc<Mutex<Vec<f32>>>) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device");
        let config = device.default_input_config().expect("no input config");
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    if recording.load(Ordering::Relaxed) {
                        let mono: Vec<f32> = data
                            .chunks(channels)
                            .map(|c| c.iter().sum::<f32>() / channels as f32)
                            .collect();
                        audio_buf
                            .lock()
                            .unwrap()
                            .extend(resample(&mono, sample_rate, SAMPLE_RATE));
                    }
                },
                |e| eprintln!("audio error: {e}"),
                None,
            )
            .expect("failed to build input stream");
        stream.play().expect("failed to start stream");
        loop {
            std::thread::park();
        }
    });
}
