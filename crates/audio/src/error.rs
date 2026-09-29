use cpal::{Error as CpalError, SampleFormat};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio input device found")]
    NoInputDevice,

    #[error("Audio input device '{0}' not found")]
    DeviceNotFound(String),

    #[error("CPAL error: {0}")]
    Cpal(#[from] CpalError),

    #[error("Resampling failed: {0}")]
    Resample(String),

    #[error("Audio recording is already in progress")]
    AlreadyRecording,

    #[error("No audio recording is currently in progress")]
    NotRecording,

    #[error("Unsupported sample format: {0:?}")]
    UnsupportedSampleFormat(SampleFormat),
}
