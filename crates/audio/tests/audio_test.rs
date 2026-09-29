use audio::Audio;

#[test]
fn test_convert_channel_to_mono_stereo() {
    let stereo = vec![0.5, 0.5, 1.0, 0.0, -0.5, 0.5];
    let mono = Audio::convert_channel_to_mono(&stereo, 2);
    assert_eq!(mono, vec![0.5, 0.5, 0.0]);
}

#[test]
fn test_convert_channel_to_mono_already_mono() {
    let original = vec![0.1, 0.2, 0.3];
    let mono = Audio::convert_channel_to_mono(&original, 1);
    assert_eq!(mono, original);
}

#[test]
fn test_convert_channel_to_mono_zero_channels() {
    let original = vec![0.1, 0.2, 0.3];
    let mono = Audio::convert_channel_to_mono(&original, 0);
    assert_eq!(mono, original);
}

#[test]
fn test_convert_channel_to_mono_empty() {
    let empty: Vec<f32> = vec![];
    let mono = Audio::convert_channel_to_mono(&empty, 2);
    assert!(mono.is_empty());
}

#[test]
fn test_resample_empty() {
    let empty: Vec<f32> = vec![];
    let res = Audio::resample(empty, 48000.0).expect("resample empty failed");
    assert!(res.is_empty());
}

#[test]
fn test_resample_already_16k_bypasses() {
    let samples = vec![0.1, 0.2, 0.3, 0.4];
    let res = Audio::resample(samples.clone(), 16000.0).expect("resample 16k failed");
    assert_eq!(res, samples);
}

#[test]
fn test_resample_48k_to_16k() {
    // 4800 samples at 48kHz is 0.1s. Resampled to 16kHz it should produce ~1600 samples.
    let samples: Vec<f32> = (0..4800)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin())
        .collect();
    let res = Audio::resample(samples, 48000.0).expect("resampling failed");
    // Due to sinc filter delay and boundary padding, the length will be approximately 1600.
    assert!(!res.is_empty());
    assert!((res.len() as i32 - 1600).abs() < 100);
}

#[test]
fn test_audio_init_and_state() {
    if let Ok(mut audio) = Audio::new() {
        assert!(!audio.is_recording());
        audio.cancel();
        assert!(!audio.is_recording());
        assert!(audio.sample_rate() > 0);
        assert!(audio.channels() > 0);
    }

    if let Ok(devices) = Audio::available_devices() {
        // Just verify it queries successfully without panicking
        println!("Available devices: {:?}", devices);
    }
}

fn assert_send<T: Send>() {}

#[test]
fn test_audio_is_send() {
    assert_send::<Audio>();
}
