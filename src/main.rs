use audio_general::audio::audio_state::{AudioState, SpectrumType};
use audio_general::audio::util::get_file;
use audio_general::wgpu::visualizer::run_visualizer;
use pollster;

use rtrb::RingBuffer;
use std::sync::{Arc, Mutex};
// use cpal::default_host

// fn main() {
//     let (samples, sample_rate) = get_file();

//     let mut audio_state = AudioState::new(SpectrumType::Frequency, samples, sample_rate);

//     pollster::block_on(run_visualizer(audio_state));
// }

//! Visualizer works
// fn main() {
//     let (samples, sample_rate) = get_file();

//     let mut audio_state = AudioState::new(SpectrumType::Frequency, samples, sample_rate);

//     pollster::block_on(run_visualizer(audio_state));
// }
