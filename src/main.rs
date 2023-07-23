use audio_general::audio::audio_state::{AudioState, SpectrumType};
use audio_general::audio::util::get_file;

use audio_general::wgpu::visualizer::run_visualizer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::sync::{Arc, Mutex};

fn main() {
    let (samples, sample_rate) = get_file();

    let audio_state = Arc::new(Mutex::new(AudioState::new(
        SpectrumType::Time,
        samples,
        sample_rate,
    )));

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let supported_config: cpal::SupportedStreamConfig = device.default_output_config().unwrap();
    let config = cpal::StreamConfig {
        channels: 2,
        buffer_size: cpal::BufferSize::Default,
        sample_rate: cpal::SampleRate(sample_rate),
    };
    match supported_config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(device, config.into(), audio_state),
        _ => unimplemented!(),
    }
}

fn run<T: cpal::Sample>(
    device: cpal::Device,
    config: cpal::StreamConfig,
    audio_state: Arc<Mutex<AudioState>>,
) {
    let audio_state_stream = Arc::clone(&audio_state);
    let (tx, rx) = std::sync::mpsc::channel();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Lock the audio_state for this scope
                let mut audio_state = audio_state_stream.lock().unwrap();

                for sample in data.iter_mut() {
                    // Get the next sample from the AudioState
                    let value = audio_state.get_sample().unwrap();
                    *sample = value;
                    let _ = tx.send(value);
                }
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            Some(std::time::Duration::from_secs(1)),
        )
        .unwrap();
    stream.play().unwrap();
    println!("Stream was built");

    pollster::block_on(run_visualizer(audio_state, rx));

    // Keep the main thread alive until you want to stop.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

// // ! Visualizer works
// fn not_main() {
//     let (samples, sample_rate) = get_file();

//     let mut audio_state = AudioState::new(SpectrumType::Frequency, samples, sample_rate);

//
// }
