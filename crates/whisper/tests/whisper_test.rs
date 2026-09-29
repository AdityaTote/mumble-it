use std::path::Path;
use whisper::{STTEngine, WhisperError};

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

#[test]
fn test_engine_is_send_and_sync() {
    assert_send::<STTEngine>();
    assert_sync::<STTEngine>();
}

#[test]
fn test_model_not_found() {
    let result = STTEngine::new("path/that/does/not/exist/model.bin");
    match result {
        Err(WhisperError::ModelNotFound(p)) => {
            assert_eq!(p, Path::new("path/that/does/not/exist/model.bin"));
        }
        _ => panic!("Expected ModelNotFound error, got {:?}", result.err()),
    }
}

#[test]
fn test_empty_buffer_error() {
    let model_paths = [
        "../../models/ggml-base.en.bin",
        "../models/ggml-base.en.bin",
        "models/ggml-base.en.bin",
    ];

    let existing_path = model_paths.iter().find(|p| Path::new(p).exists());

    if let Some(path) = existing_path {
        let engine = STTEngine::new(path).expect("failed to load model");
        let result = engine.transcribe(&[]);
        match result {
            Err(WhisperError::EmptyBuffer) => {}
            _ => panic!("Expected EmptyBuffer error, got {:?}", result),
        }
    }
}

#[test]
fn test_transcribe_silence() {
    let model_paths = [
        "../../models/ggml-base.en.bin",
        "../models/ggml-base.en.bin",
        "models/ggml-base.en.bin",
    ];

    let existing_path = model_paths.iter().find(|p| Path::new(p).exists());

    if let Some(path) = existing_path {
        let engine = STTEngine::new(path).expect("failed to load model");
        // 1 second of 16kHz mono silence
        let silence = vec![0.0f32; 16000];
        let result = engine.transcribe(&silence);
        assert!(result.is_ok(), "transcription failed: {:?}", result.err());
    }
}
