use std::path::Path;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::WhisperError;

pub struct STTEngine {
    whisper_ctx: WhisperContext,
}

impl STTEngine {
    /// Initialize the Whisper STT engine from a model file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, WhisperError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(WhisperError::ModelNotFound(path_ref.to_path_buf()));
        }

        let ctx = WhisperContext::new_with_params(path_ref, WhisperContextParameters::default())
            .map_err(WhisperError::ContextInitializationFailed)?;

        Ok(Self { whisper_ctx: ctx })
    }

    /// Transcribe 16 kHz mono f32 PCM audio samples to text.
    pub fn transcribe(&self, buffer: &[f32]) -> Result<String, WhisperError> {
        if buffer.is_empty() {
            return Err(WhisperError::EmptyBuffer);
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        Self::load_default_params(&mut params);

        self.transcribe_with_params(buffer, params)
    }

    /// Transcribe audio with custom Whisper parameters.
    pub fn transcribe_with_params(
        &self,
        buffer: &[f32],
        params: FullParams,
    ) -> Result<String, WhisperError> {
        if buffer.is_empty() {
            return Err(WhisperError::EmptyBuffer);
        }

        let mut state = self
            .whisper_ctx
            .create_state()
            .map_err(WhisperError::StateCreationFailed)?;

        state
            .full(params, buffer)
            .map_err(WhisperError::InferenceFailed)?;

        let number_of_segments = state.full_n_segments();
        let mut text = String::new();

        for i in 0..number_of_segments {
            if let Some(segment) = state.get_segment(i) {
                let segment_text = segment
                    .to_str()
                    .map_err(WhisperError::SegmentExtractionFailed)?;
                text.push_str(segment_text);
            }
        }

        Ok(text)
    }

    fn load_default_params(params: &mut FullParams) {
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
    }
}
