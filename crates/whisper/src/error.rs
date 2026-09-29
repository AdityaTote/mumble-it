use std::path::PathBuf;
use thiserror::Error;
use whisper_rs::WhisperError as WhisperRsError;

#[derive(Debug, Error)]
pub enum WhisperError {
    #[error("Model file not found at path: {0}")]
    ModelNotFound(PathBuf),

    #[error("Failed to initialize Whisper context: {0}")]
    ContextInitializationFailed(WhisperRsError),

    #[error("Failed to create Whisper state: {0}")]
    StateCreationFailed(WhisperRsError),

    #[error("Audio buffer is empty")]
    EmptyBuffer,

    #[error("Whisper inference execution failed: {0}")]
    InferenceFailed(WhisperRsError),

    #[error("Failed to decode transcribed text segment: {0}")]
    SegmentExtractionFailed(WhisperRsError),
}
