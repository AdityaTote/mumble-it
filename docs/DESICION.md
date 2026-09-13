# Design Decisions

This document outlines the architectural choices and model evaluations for **Mumble-It**.

## Why Another Whisper Pipeline?

Mumble-It is built to be an accessible, local-first speech assistant that transforms a typing-heavy workflow into a speech-driven one.

While many speech-to-text (STT) solutions exist, most rely on external (exogenous) cloud backends. Mumble-It focuses on:
- Bringing STT execution directly onto your local machine.
- Operating independently of third-party cloud services and external backends.
- Ensuring privacy, low latency, and full local control.

## Why the Rust Ecosystem?

Rust was chosen as the implementation language for several key reasons:

- **Lightweight Footprint**: Offers a significantly smaller memory and binary footprint compared to typical desktop application runtimes.
- **Background Efficiency**: Well-suited for background applications that need to stay resident with minimal CPU and RAM overhead.
- **Open-Weight Model Support**: Strong ecosystem support for running open-weight models, specifically `whisper.cpp` through native Rust bindings (`whisper-rs`).

## Speech-to-Text (STT) Models & Engines

### 1. `whisper.cpp` (Selected)
- **Status**: Selected engine.
- **Rationale**: Features direct Rust integration via `whisper-rs`, exposing high-performance Rust FFI bindings to interact directly with `whisper.cpp`. It enables efficient local execution of Whisper models.

### 2. Faster-Whisper (`whisper-fast`)
- **Status**: Evaluated.
- **Rationale**: While performant and based on the same model weights, it is primarily developed and exposed for Python. Integrating it into a native Rust application would introduce heavy Python runtime dependencies.

### 3. Moonshine
- **Status**: Deferred (future scope).
- **Rationale**: Moonshine presents a compelling, efficient alternative to Whisper. However, it currently lacks maintained, production-ready Rust support (an existing community crate, `moonshine-rs`, is unmaintained and lacks proper support).
- **Future Plan**: Kept as a future roadmap item to implement a dedicated native FFI layer for Moonshine.

---

## Related Documents
- [Architecture & Pipeline](ARCHITECTURE.md): Full pipeline breakdown from Hyprland keybind to text injection.
