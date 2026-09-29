use std::sync::{Arc, Mutex};

use cpal::{
    Device, FromSample, Sample, SampleFormat, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, WindowFunction,
    audioadapter_buffers::owned::InterleavedOwned,
};

use crate::error::AudioError;

pub struct Audio {
    device: Device,
    sample_format: SampleFormat,
    config: StreamConfig,
    audio_buffer: Option<Arc<Mutex<Vec<f32>>>>,
    stream: Option<Stream>,
}

impl Audio {
    /// Initialize audio capture using the system default input device.
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;
        Self::with_device(device)
    }

    /// Initialize audio capture using a specific input device.
    pub fn with_device(device: Device) -> Result<Self, AudioError> {
        let supported_config = device.default_input_config()?;
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.config();

        Ok(Self {
            device,
            sample_format,
            config,
            audio_buffer: None,
            stream: None,
        })
    }

    /// Initialize audio capture using an input device name.
    pub fn with_device_name(name: &str) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let mut devices = host.input_devices()?;
        let device = devices
            .find(|dev| {
                dev.description()
                    .map(|d| d.name() == name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| AudioError::DeviceNotFound(name.to_string()))?;
        Self::with_device(device)
    }

    /// Enumerate names of all available audio input devices.
    pub fn available_devices() -> Result<Vec<String>, AudioError> {
        let host = cpal::default_host();
        let devices = host.input_devices()?;
        let mut names = Vec::new();
        for dev in devices {
            if let Ok(desc) = dev.description() {
                names.push(desc.name().to_string());
            }
        }
        Ok(names)
    }

    /// Returns the input device sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Returns the number of input channels.
    pub fn channels(&self) -> u16 {
        self.config.channels
    }

    /// Returns `true` if audio capture is currently running.
    pub fn is_recording(&self) -> bool {
        self.stream.is_some()
    }

    /// Start capturing audio into the in-memory buffer.
    pub fn start(&mut self) -> Result<(), AudioError> {
        if self.stream.is_some() {
            return Err(AudioError::AlreadyRecording);
        }

        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let stream = self.build_stream(&buffer)?;

        stream.play()?;

        self.stream = Some(stream);
        self.audio_buffer = Some(buffer);
        Ok(())
    }

    /// Stop capturing audio, downmix channels to mono, resample to 16 kHz, and return the PCM samples.
    pub fn stop(&mut self) -> Result<Vec<f32>, AudioError> {
        if self.stream.is_none() {
            return Err(AudioError::NotRecording);
        }

        // Drop the stream immediately to halt callback writes
        drop(self.stream.take());

        let raw_buffer = self
            .audio_buffer
            .take()
            .map(|buff| {
                buff.lock()
                    .map(|mut b| std::mem::take(&mut *b))
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        if raw_buffer.is_empty() {
            return Ok(Vec::new());
        }

        // Downmix channels to mono
        let channels = self.config.channels as usize;
        let mono = Self::convert_channel_to_mono(&raw_buffer, channels);

        // Resample mono PCM to 16 kHz
        let sample_rate = self.config.sample_rate as f64;
        let final_buffer = Self::resample(mono, sample_rate)?;

        Ok(final_buffer)
    }

    /// Cancel the current recording and discard buffered audio without resampling.
    pub fn cancel(&mut self) {
        drop(self.stream.take());
        self.audio_buffer.take();
    }

    // writing the mic audio to buffer
    fn receive_audio<T: Sample>(data: &[T], buffer: &Arc<Mutex<Vec<f32>>>)
    where
        f32: FromSample<T>,
    {
        if let Ok(mut buf) = buffer.lock() {
            buf.extend(data.iter().map(|&d| d.to_sample::<f32>()));
        }
    }

    // building the stream according to input data format
    fn build_stream(&self, audio_buffer: &Arc<Mutex<Vec<f32>>>) -> Result<Stream, AudioError> {
        let err_fn = |err| eprintln!("audio input stream error: {}", err);
        let buffer = Arc::clone(audio_buffer);
        let config = self.config;

        let stream = match self.sample_format {
            SampleFormat::F32 => self.device.build_input_stream(
                config,
                move |data: &[f32], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::I16 => self.device.build_input_stream(
                config,
                move |data: &[i16], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::U16 => self.device.build_input_stream(
                config,
                move |data: &[u16], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::I32 => self.device.build_input_stream(
                config,
                move |data: &[i32], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::U32 => self.device.build_input_stream(
                config,
                move |data: &[u32], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::I8 => self.device.build_input_stream(
                config,
                move |data: &[i8], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::U8 => self.device.build_input_stream(
                config,
                move |data: &[u8], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            SampleFormat::F64 => self.device.build_input_stream(
                config,
                move |data: &[f64], _| Self::receive_audio(data, &buffer),
                err_fn,
                None,
            )?,
            format => return Err(AudioError::UnsupportedSampleFormat(format)),
        };

        Ok(stream)
    }

    /// Downmix multi-channel audio buffer to a single mono channel.
    pub fn convert_channel_to_mono(audio_buffer: &[f32], channels: usize) -> Vec<f32> {
        if channels <= 1 {
            return audio_buffer.to_vec();
        }
        if audio_buffer.is_empty() {
            return Vec::new();
        }

        let mut mono = Vec::with_capacity(audio_buffer.len() / channels);
        for frame in audio_buffer.chunks_exact(channels) {
            let sum: f32 = frame.iter().sum();
            mono.push(sum / channels as f32);
        }
        mono
    }

    /// Resample mono PCM audio from `input_sample_rate` to 16 kHz for Whisper input.
    pub fn resample(
        input_buffer: Vec<f32>,
        input_sample_rate: f64,
    ) -> Result<Vec<f32>, AudioError> {
        if input_buffer.is_empty() {
            return Ok(Vec::new());
        }

        let target_sample_rate = 16000.0;
        // If already 16 kHz (within 1 Hz), bypass resampling
        if (input_sample_rate - target_sample_rate).abs() < 1.0 {
            return Ok(input_buffer);
        }

        let ratio = target_sample_rate / input_sample_rate;
        let channels = 1;
        let chunk_size = 1024;
        let input_frames = input_buffer.len();

        let sinc_params = SincInterpolationParameters::new(128, WindowFunction::BlackmanHarris2);

        let mut resampler = Async::<f32>::new_sinc(
            ratio,
            2.0,
            &sinc_params,
            chunk_size,
            channels,
            FixedAsync::Input,
        )
        .map_err(|e| AudioError::Resample(e.to_string()))?;

        let input_adapter = InterleavedOwned::new_from(input_buffer, channels, input_frames)
            .map_err(|e| AudioError::Resample(e.to_string()))?;

        let output_adapter = resampler
            .process_all(&input_adapter, input_frames, None)
            .map_err(|e| AudioError::Resample(e.to_string()))?;

        Ok(output_adapter.take_data())
    }
}
