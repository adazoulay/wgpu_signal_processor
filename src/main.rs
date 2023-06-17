use audio_general::audio::audio_state::{AudioState, SpectrumType};
use audio_general::audio::util::get_file;
use audio_general::wgpu::visualizer::run;
use pollster;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::time::Duration;

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Create an audio output stream
    // let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // // Create an AudioState
    // let (samples, sample_rate) = get_file();
    // let audio_state = Arc::new(Mutex::new(AudioState::new(
    //     SpectrumType::Frequency,
    //     samples,
    //     sample_rate,
    // )));

    // // Clone the AudioState to use in the separate thread
    // let audio_state_for_thread = Arc::clone(&audio_state);

    // // Create a separate thread to handle FFT computation and visualization updates
    // let fft_thread = thread::spawn(move || {
    //     let slice_size = audio_state_for_thread.lock().unwrap().slice_size;
    //     while !audio_state_for_thread.lock().unwrap().samples.is_empty() {
    //         // Get a chunk of samples
    //         let samples_chunk = audio_state_for_thread.lock().unwrap().get_next_slice();

    //         // Compute the FFT and update the visualization
    //         // Replace with your actual FFT computation and visualization update logic
    //         println!("Samples chunk: {:?}", samples_chunk);
    //         thread::sleep(Duration::from_millis(10));
    //     }
    // });

    // // Start playing the audio
    // let audio_sink = rodio::Sink::try_new(&stream_handle).unwrap();
    // audio_sink.append(audio_state.lock().unwrap().clone());
    // audio_sink.sleep_until_end();

    // // Wait for the FFT thread to finish
    // fft_thread.join().unwrap();
    let (samples, sample_rate) = get_file();
    let audio_state = AudioState::new(SpectrumType::Frequency, samples, sample_rate);
    pollster::block_on(run(audio_state));
}
