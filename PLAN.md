# Dictator (Dictation app for macOS)

Lightweight push-to-talk dictation for macOS, built in Rust on top of whisper.cpp.
A focused alternative to MacWhisper / WhisperFlow — no subscriptions, no bloat.

## Why

MacWhisper is a kitchen sink. This does one thing: press button, speak, get text.
Open source. Patreon if anyone cares.

## MVP Scope

- Global hotkey (push-to-talk): hold to record, release to transcribe and insert
- Text inserted into any active text field via macOS Accessibility API
- Spoken punctuation via configurable word→symbol mapping
- TOML config file, no GUI
- Languages: English, Russian, Ukrainian

## Stack

| Concern           | Crate                                   |
| ----------------- | --------------------------------------- |
| Whisper inference | `whisper-rs` (bindings for whisper.cpp) |
| Audio capture     | `cpal` (Core Audio on macOS)            |
| Global hotkey     | `rdev`                                  |
| Text insertion    | `enigo` + CGEvent fallback              |
| Config            | `toml` + `serde`                        |
| Async runtime     | `tokio` (model load/unload timer)       |
| History (future)  | `rusqlite`                              |

## Pipeline

```
[hotkey pressed]  → cpal starts recording into Vec<f32> buffer
[hotkey released] → stop recording
                  → whisper-rs inference
                  → post-processing (punctuation map)
                  → enigo inserts text into active field
```

## Model

- Format: whisper.cpp `.bin` (NOT CoreML/WhisperKit — different format, no Rust bindings)
- MacWhisper uses WhisperKit (CoreML) — incompatible, cannot reuse
- Need to download separately: `ggml-large-v3-turbo.bin` (~1.6GB) from Hugging Face
- Target: `large-v3-turbo` — 6x faster than large-v3, nearly same quality, good for push-to-talk latency
- Future: `whisper-dictate download large-v3-turbo` CLI command

### Memory Management

Model is 1.5GB — too heavy to keep in memory permanently for an occasional-use tool.

Strategy: lazy load + timeout unload

```
[hotkey pressed] → model not loaded? → load first (slight delay, acceptable)
[text inserted]  → start idle timer
[timer expires]  → unload model from memory (free 1.5GB)
[hotkey pressed] → load again
```

Implementation: `Arc<Mutex<Option<WhisperContext>>>` + `tokio::time::sleep`

## Config (TOML)

```toml
[general]
hotkey = "F13"

[model]
path = "~/Downloads/ggml-large-v3-turbo.bin"
unload_timeout_secs = 300  # 0 = never unload

[whisper]
language = "auto"
initial_prompt = "Текст українською, по-русски, in English."

[punctuation]
enabled = true
map = [
    ["запятая", ","],
    ["точка", "."],
    ["вопросительный знак", "?"],
    ["восклицательный знак", "!"],
    ["двоеточие", ":"],
    ["тире", "—"],
    ["новая строка", "\n"],
    ["кома", ","],
    ["крапка", "."],
    ["знак питання", "?"],
]
```

## Recording Indicator

macOS shows an orange microphone indicator in the menu bar automatically when any app
accesses the microphone — this is an OS-level feature, no implementation needed.

Future (post-MVP): custom floating overlay (NSPanel) with audio waveform animation,
similar to MacWhisper's blue bubble. Requires AppKit via `objc2` crate or thin Swift wrapper.

## Key Decisions

- **No VAD** — push-to-talk eliminates the need entirely
- **No GUI** — config file only
- **Lazy model loading** — load on first use, unload after idle timeout
- **One hotkey** — no per-language bindings, `initial_prompt` handles language hint
- **Multilingual model** — handles EN/RU/UK + code-switching (e.g. "Redis кластер")
- **whisper.cpp format** — CoreML/WhisperKit not viable (no Rust bindings)

## Not in MVP

- Model download command (manual download for now)
- Dictation history via SQLite
- Fine-tuning for Ukrainian
- Hardware button / HID remote (fun future idea — like Alexa remote)
- Auto-update
- GUI / menubar app
- Cross-platform compatibility.

## Possible Future Monetization

- GitHub + Patreon (honest model)
- No subscriptions, no paywalled features

## Future Direction: Mobile

**Concept:** iOS app with push-to-talk dictation + auto language detection (EN/RU/UK).
Target audience: people who text in multiple languages and hate switching keyboards.

**Business model: Open Core**

- Desktop (Dictator) — open source, free, GitHub. Serves as marketing / trust builder.
- Mobile — paid, subscription. Justified by inference costs if cloud, or App Store overhead if on-device.

**Two implementation paths:**

- On-device: Core ML Whisper (Apple's Neural Engine), no server costs, one-time purchase viable
- Cloud: stream audio to server, run whisper there, return text. Subscription model, lower barrier to ship.

**Distribution:** TestFlight for personal use and beta — bypasses App Store review cycles entirely.
App Store only when/if product is ready and the $99/year fee makes sense.
