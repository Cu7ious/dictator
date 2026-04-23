use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use rdev::{listen, Event, EventType, Key as RdevKey};

use crate::sounds;

pub fn spawn_listener(
    recording: Arc<AtomicBool>,
    audio_buf: Arc<Mutex<Vec<f32>>>,
    audio_tx: Sender<Vec<f32>>,
    icon_tx: Sender<bool>,
    hotkey: RdevKey,
    sound_start: String,
    sound_stop: String,
    sounds_enabled: bool,
    record_delay_ms: u64,
) {
    std::thread::spawn(move || {
        listen(move |event: Event| match event.event_type {
            EventType::KeyPress(k) if k == hotkey => {
                if !recording.load(Ordering::Relaxed) {
                    audio_buf.lock().unwrap().clear();
                    if sounds_enabled {
                        sounds::play_start(&sound_start);
                    }
                    icon_tx.send(true).ok();
                    // Open mic after delay so start sound isn't captured
                    let recording = Arc::clone(&recording);
                    let delay = std::time::Duration::from_millis(record_delay_ms);
                    std::thread::spawn(move || {
                        std::thread::sleep(delay);
                        recording.store(true, Ordering::Relaxed);
                    });
                }
            }
            EventType::KeyRelease(k) if k == hotkey => {
                recording.store(false, Ordering::Relaxed);
                let audio = std::mem::take(&mut *audio_buf.lock().unwrap());
                if sounds_enabled {
                    sounds::play_stop(&sound_stop);
                }
                icon_tx.send(false).ok();
                audio_tx.send(audio).ok();
            }
            _ => {}
        })
        .expect("failed to listen for global events (check Accessibility permissions)");
    });
}
