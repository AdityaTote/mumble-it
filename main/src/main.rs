use std::{
    path::Path,
    sync::{Arc, Mutex},
};

// take an audio file
// send it to the whisper model
// grab the text and show tf
use cpal::{
    Device, FromSample, Sample, SampleFormat, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, WindowFunction,
    audioadapter_buffers::owned::InterleavedOwned,
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn receive_audio<T: Sample>(data: &[T], buffer: &Arc<Mutex<Vec<f32>>>)
where
    f32: FromSample<T>,
{
    // lock buffer
    let mut buf = buffer.lock().unwrap();

    // convert data to f32
    for &d in data {
        // convert typeof d to f32
        let new_d: f32 = d.to_sample::<f32>();

        // push data to buffer
        buf.push(new_d);
    }
}

fn get_stream(
    sample_format: SampleFormat,
    device: &Device,
    config: StreamConfig,
    audio_buffer: &Arc<Mutex<Vec<f32>>>,
) -> Stream {
    let err_fn = |err| eprintln!("error occur on i/p and o/p stream: {}", err);

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[f32], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[i16], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[u16], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),
        SampleFormat::I32 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[i32], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),

        SampleFormat::U32 => device.build_input_stream(
            config.clone(),
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[u32], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),

        SampleFormat::I8 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[i8], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),

        SampleFormat::U8 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[u8], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),

        SampleFormat::F64 => device.build_input_stream(
            config,
            {
                let buffer = Arc::clone(&audio_buffer);
                move |data: &[f64], _| receive_audio(data, &buffer)
            },
            err_fn,
            None,
        ),

        format => panic!("Unsupported sample format: {:?}", format),
    }
    .unwrap();
    stream
}

fn convert_channel_to_mono(audio_buffer: Arc<Mutex<Vec<f32>>>, channels: usize) -> Vec<f32> {
    let buffers = audio_buffer.lock().unwrap();
    let mut mono = Vec::with_capacity(buffers.len() / channels);

    for frame in buffers.chunks_exact(channels) {
        let sum: f32 = frame.iter().sum();
        mono.push(sum / channels as f32);
    }
    mono
}

fn resample(
    input_buffer: Vec<f32>,
    input_sample_rate: f64,
) -> Result<InterleavedOwned<f32>, Box<dyn std::error::Error>> {
    let target_sample_rate = 16000; // 16kHz
    let ratio = target_sample_rate as f64 / input_sample_rate;
    let channels = 1;
    let chunk_size = 1024;
    let input_frames = input_buffer.len();

    // sin interpolation params
    let params = SincInterpolationParameters::new(128, WindowFunction::BlackmanHarris2);

    // resampler with sinc (async)
    let mut resampler =
        Async::<f32>::new_sinc(ratio, 2.0, &params, chunk_size, channels, FixedAsync::Input)?;

    // input adapter -> for teaching model for input buffer
    let input_adapter = InterleavedOwned::new_from(input_buffer, channels, input_frames)?;

    // resampled buffer
    let output_adapter = resampler.process_all(&input_adapter, input_frames, None)?;

    Ok(output_adapter)
}

fn generate_text(_model_path: &Path, whisper_ctx: WhisperContext, sample: Vec<f32>) -> String {
    let mut state = whisper_ctx.create_state().expect("failed to create state");
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_initial_prompt("experience");
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let _ = state.full(params, &sample);
    let num_of_segments = state.full_n_segments();
    let mut text = String::new();
    for i in 0..num_of_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(segment_text) = segment.to_str() {
                text.push_str(segment_text);
            }
        }
    }
    text
}

fn main() {
    let model_path = if Path::new("ggml-medium.en.bin").exists() {
        Path::new("models/ggml-base.en.bin")
    } else if Path::new("../models/ggml-base.en.bin").exists() {
        Path::new("../models/ggml-base.en.bin")
    } else {
        panic!("Could not find ggml-base.en.bin in 'models/' or '../models/'");
    };
    let whisper_ctx =
        WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .expect("failed to load model");
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let mut supported_config_range = device
        .supported_input_configs()
        .expect("error while configuration");
    let supported_config = supported_config_range
        .next()
        .expect("no supported configs")
        .with_max_sample_rate();
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.config();
    println!("Sample format: {:?}", sample_format);
    println!("Channels: {}", config.channels);
    println!("Sample rate: {}", config.sample_rate);
    println!("Config: {:?}", config);
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let stream = get_stream(sample_format, &device, config, &audio_buffer);

    stream.play().unwrap();

    println!("Recording for 10 seconds...");

    std::thread::sleep(std::time::Duration::from_secs(10));

    stream.pause().unwrap();

    // channel conversation
    let mono_buffer = convert_channel_to_mono(audio_buffer, config.channels as usize);

    // resampler
    let samples = resample(mono_buffer, config.sample_rate as f64)
        .unwrap()
        .take_data();

    let text = generate_text(model_path, whisper_ctx, samples);
    println!("{:?}", text)
}
