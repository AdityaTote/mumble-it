# mumble-it

> A minimalist, local speech-to-text pipeline in Rust.

`mumble-it` captures audio from your microphone, downmixes and resamples it to 16 kHz in real-time, and transcribes your speech locally using OpenAI's Whisper model via `whisper-rs`.

---

## 🎙️ Pipeline

```
Microphone (CPAL)
       │
       ▼
Mono Downmix
       │
       ▼
Resample to 16 kHz (Rubato)
       │
       ▼
Whisper Inference (whisper-rs / GGML)
       │
       ▼
Transcribed Text
```

---

## 🚀 Getting Started

### 1. Prerequisites

Ensure you have Rust and standard audio development libraries installed:

**Linux (Ubuntu / Debian):**
```bash
sudo apt update && sudo apt install -y libasound2-dev pkg-config cmake clang
```

### 2. Download Whisper Model

Place a Whisper GGML model in the `models/` directory:

```bash
mkdir -p models
curl -L -o models/ggml-base.en.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
```

### 3. Run

```bash
cargo run --manifest-path main/Cargo.toml
```

Speak into your microphone during the 10-second recording window to receive your transcription in the terminal.

---

## 🛠️ Built With

- **[CPAL](https://github.com/RustAudio/cpal)** — Cross-platform microphone capture and stream handling.
- **[Rubato](https://github.com/HEnquist/rubato)** — High-performance asynchronous audio resampling.
- **[Whisper-rs](https://github.com/tazz4843/whisper-rs)** — Rust bindings for `whisper.cpp`.

---

## 📄 License

MIT
