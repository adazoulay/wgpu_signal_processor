use audio_general::audio::audio_state::{AudioState, SpectrumType};
use audio_general::audio::util::get_file;
use audio_general::wgpu::visualizer::run;
use pollster;

use rtrb::RingBuffer;
use std::sync::{Arc, Mutex};
// use cpal::default_host

fn main() {
    let (samples, sample_rate) = get_file();

    let audio_state = AudioState::new(SpectrumType::Time, samples, sample_rate);

    // let audio_state = Arc::new(Mutex::new(AudioState::new(
    //     SpectrumType::Frequency,
    //     samples,
    //     sample_rate,
    // )));

    let (mut producer, mut consumer) = RingBuffer::<f32>::new(audio_state.size);

    // pollster::block_on(run(audio_state));

    for sample in audio_state.clone() {
        producer.push(sample).unwrap();
    }

    audio_out(consumer);
}

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Data;
use cpal::{FromSample, Sample};

fn audio_out(mut consumer: rtrb::Consumer<f32>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("No supported config")
        .with_max_sample_rate();

    let config = supported_config.into();

    println!("{:?}", consumer);
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                println!("Starting loop");
                println!("Data length: {}", data.len());
                for sample in data.iter_mut() {
                    println!("{}", sample);
                    *sample = consumer.pop().unwrap_or(0.0);
                }
            },
            move |err| println!("{:?}", err),
            None,
        )
        .expect("Stream built");

    stream.play().unwrap();
}
