use std::path::PathBuf;

use rdev::Key as RdevKey;
use serde::Deserialize;

// ── Structs ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub whisper: WhisperConfig,
    #[serde(default)]
    pub sounds: SoundsConfig,
}

#[derive(Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { hotkey: default_hotkey() }
    }
}

fn default_hotkey() -> String { "AltGr".into() }

#[derive(Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_min_duration_secs")]
    pub min_duration_secs: f32,
    #[serde(default = "default_silence_threshold")]
    pub silence_threshold: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            min_duration_secs: default_min_duration_secs(),
            silence_threshold: default_silence_threshold(),
        }
    }
}

fn default_min_duration_secs() -> f32 { 0.5 }
fn default_silence_threshold() -> f32 { 0.01 }

#[derive(Deserialize)]
pub struct ModelConfig {
    #[serde(default)]
    pub path: String,
    #[serde(default = "default_unload_timeout_mins")]
    pub unload_timeout_mins: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self { path: String::new(), unload_timeout_mins: default_unload_timeout_mins() }
    }
}

fn default_unload_timeout_mins() -> u64 { 10 }

#[derive(Deserialize)]
pub struct WhisperConfig {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_initial_prompt")]
    pub initial_prompt: String,
    #[serde(default = "default_beam_size")]
    pub beam_size: i32,
    #[serde(default = "default_suppress_blank")]
    pub suppress_blank: bool,
    #[serde(default = "default_no_speech_threshold")]
    pub no_speech_threshold: f32,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            initial_prompt: default_initial_prompt(),
            beam_size: default_beam_size(),
            suppress_blank: default_suppress_blank(),
            no_speech_threshold: default_no_speech_threshold(),
        }
    }
}

fn default_language() -> String { "auto".into() }
fn default_initial_prompt() -> String { "Текст українською, по-русски, in English.".into() }
fn default_beam_size() -> i32 { 5 }
fn default_suppress_blank() -> bool { true }
fn default_no_speech_threshold() -> f32 { 0.6 }

#[derive(Deserialize)]
pub struct SoundsConfig {
    #[serde(default = "default_sounds_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sound_start")]
    pub start: String,
    #[serde(default = "default_sound_stop")]
    pub stop: String,
    #[serde(default = "default_record_delay_ms")]
    pub record_delay_ms: u64,
}

impl Default for SoundsConfig {
    fn default() -> Self {
        Self {
            enabled: default_sounds_enabled(),
            start: default_sound_start(),
            stop: default_sound_stop(),
            record_delay_ms: default_record_delay_ms(),
        }
    }
}

fn default_sounds_enabled() -> bool { true }
fn default_sound_start() -> String { "/System/Library/Sounds/Ping.aiff".into() }
fn default_sound_stop() -> String { "/System/Library/Sounds/Pop.aiff".into() }
fn default_record_delay_ms() -> u64 { 300 }

// ── Loading ───────────────────────────────────────────────────────────────────

pub fn load() -> Config {
    // (1) CWD override for development
    let cwd = std::env::current_dir().ok().map(|d| d.join("config.toml"));
    if let Some(ref p) = cwd {
        if p.exists() {
            return parse(p);
        }
    }

    // (2) User config
    let user = user_config_path();
    if !user.exists() {
        let dir = user.parent().unwrap();
        std::fs::create_dir_all(dir).expect("failed to create config dir");
        let template = include_str!("../default_config.toml");
        std::fs::write(&user, template).expect("failed to write default config");
        println!("Created config at {}", user.display());
    }
    parse(&user)
}

fn parse(path: &PathBuf) -> Config {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    toml::from_str(&raw)
        .unwrap_or_else(|e| panic!("invalid config at {}: {e}", path.display()))
}

pub fn config_locations() -> Vec<(&'static str, PathBuf)> {
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .join("config.toml");
    let user = user_config_path();
    vec![
        ("./config.toml (dev override)", cwd),
        ("~/.config/dictator/config.toml (user config)", user),
    ]
}

fn user_config_path() -> PathBuf {
    dirs::config_dir()
        .expect("cannot resolve config dir")
        .join("dictator")
        .join("config.toml")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn expand_path(raw: &str) -> PathBuf {
    if raw.starts_with("~/") {
        dirs::home_dir()
            .expect("cannot resolve home dir")
            .join(&raw[2..])
    } else {
        PathBuf::from(raw)
    }
}

pub fn parse_key(s: &str) -> RdevKey {
    use RdevKey::*;
    match s {
        "Alt" => Alt,
        "AltGr" => AltGr,
        "Backspace" => Backspace,
        "CapsLock" => CapsLock,
        "ControlLeft" => ControlLeft,
        "ControlRight" => ControlRight,
        "Delete" => Delete,
        "DownArrow" => DownArrow,
        "End" => End,
        "Escape" => Escape,
        "F1" => F1,
        "F2" => F2,
        "F3" => F3,
        "F4" => F4,
        "F5" => F5,
        "F6" => F6,
        "F7" => F7,
        "F8" => F8,
        "F9" => F9,
        "F10" => F10,
        "F11" => F11,
        "F12" => F12,
        "Home" => Home,
        "Insert" => Insert,
        "MetaLeft" => MetaLeft,
        "MetaRight" => MetaRight,
        "PageDown" => PageDown,
        "PageUp" => PageUp,
        "Return" => Return,
        "RightArrow" => RightArrow,
        "ScrollLock" => ScrollLock,
        "ShiftLeft" => ShiftLeft,
        "ShiftRight" => ShiftRight,
        "Space" => Space,
        "Tab" => Tab,
        "UpArrow" => UpArrow,
        "LeftArrow" => LeftArrow,
        other => panic!("unknown hotkey \"{other}\". Valid keys: Alt, AltGr, F1–F20, ControlLeft/Right, ShiftLeft/Right, MetaLeft/Right, etc."),
    }
}
