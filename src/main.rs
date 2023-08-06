use audio_general::audio::audio_clip::{AudioClip, AudioClipEnum};
use audio_general::audio::audio_state::AudioState;
use audio_general::audio::io::AudioIO;
use audio_general::wgpu::visualizer::run_visualizer;
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use audio_general::audio::util::from_file;

fn main() {
    let audio_io = AudioIO::new();
    let sample_rate = audio_io.supported_output_config.sample_rate().0;
    
    let mut audio_state =  AudioState::<[f32;2]>::new(sample_rate);
    
    let (samples, sample_rate, channels) = from_file().unwrap();
    let audio_clip = AudioClipEnum::from_samples(samples, sample_rate, channels);
    audio_state.add_clip::<[f32;2]>(audio_clip);
    
    
    let (samples, sample_rate, channels) = audio_io.record().unwrap();
    let audio_clip = AudioClipEnum::from_samples(samples, sample_rate, channels);    
    audio_state.add_clip::<[f32;2]>(audio_clip);
    

    match audio_io.supported_output_config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(
            audio_io.output_device,
            audio_io.supported_output_config.into(),
            audio_state,
        ),
        _ => unimplemented!(),
    }
}

fn run<T: cpal::Sample>(
    device: cpal::Device,
    stream_config: cpal::StreamConfig,
    audio_state: AudioState<[f32;2]>,
) {
    let audio_metadata = audio_state.get_metadata();
    let audio_state = Arc::new(Mutex::new(audio_state));

    let (tx, rx) = std::sync::mpsc::channel();

    let stream = device
    .build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let audio_state = Arc::clone(&audio_state);
            let mut audio_state = audio_state.lock().unwrap();
            let mut data_index = 0;
            while data_index < data.len() {
                if let Some(frame) = audio_state.get_sample() {
                    for sample in frame.iter() {
                        if data_index < data.len() {
                            data[data_index] = *sample;
                            data_index += 1;
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            // Fill the rest of the buffer with silence if there is no more data.
            for i in data_index..data.len() {
                data[i] = 0.0;
            }
            let _ = tx.send(data.to_vec());
        },
        |err| eprintln!("an error occurred on stream: {}", err),
        Some(std::time::Duration::from_secs(1)),
    )
    .unwrap();
stream.play().unwrap();

    pollster::block_on(run_visualizer(audio_metadata, rx));

    // Keep the main thread alive until you want to stop.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
