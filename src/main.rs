use audio_general::audio::audio_state::AudioState;
use audio_general::audio::util::get_file;

use audio_general::wgpu::visualizer::run_visualizer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::sync::{Arc, Mutex};

struct AudioPlayer {
    _host: cpal::Host,
    device: cpal::Device,
    supported_config: cpal::SupportedStreamConfig,
    stream_config: cpal::StreamConfig,
}

impl AudioPlayer {
    fn new(sample_rate: u32) -> Self {
        let _host = cpal::default_host();
        let device = _host.default_output_device().unwrap();
        let supported_config: cpal::SupportedStreamConfig = device.default_output_config().unwrap();
        let stream_config = cpal::StreamConfig {
            channels: 2,
            buffer_size: cpal::BufferSize::Default,
            sample_rate: cpal::SampleRate(sample_rate),
        };
        Self {
            _host,
            device,
            supported_config,
            stream_config,
        }
    }
}

fn main() {
    let (samples, sample_rate) = get_file();

    let audio_state = AudioState::new(samples, sample_rate);
    let audio_player = AudioPlayer::new(sample_rate);

    match audio_player.supported_config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(
            audio_player.device,
            audio_player.stream_config.into(),
            audio_state,
        ),
        _ => unimplemented!(),
    }
}

fn run<T: cpal::Sample>(
    device: cpal::Device,
    stream_config: cpal::StreamConfig,
    audio_state: AudioState,
) {
    let audio_metadata = audio_state.get_metadata();
    let audio_state = Arc::new(Mutex::new(audio_state));

    let (tx, rx) = std::sync::mpsc::channel();

    let stream = device
        .build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let audio_state = Arc::clone(&audio_state);
                let mut chunk = Vec::with_capacity(data.len());

                let mut audio_state = audio_state.lock().unwrap();
                for sample in data.iter_mut() {
                    // Get the next sample from the AudioState
                    let value = audio_state.get_sample().unwrap();
                    *sample = value;
                    chunk.push(value);
                }
                let _ = tx.send(chunk);
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            Some(std::time::Duration::from_secs(1)),
        )
        .unwrap();
    stream.play().unwrap();
    println!("Stream was built");

    pollster::block_on(run_visualizer(audio_metadata, rx));

    // Keep the main thread alive until you want to stop.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
