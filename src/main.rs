// use audio_general::audio::wave_table::{WaveTable, WaveTableOscillator, WaveType};
// use audio_general::wgpu::
// use rodio::{OutputStream, Source};
// use std::time::Duration;
use audio_general::audio::audio_state::{AudioState, SpectrumType};
use audio_general::wgpu::visualizer::run;
use pollster;

fn main() {
    // let wave_table = WaveTable::new(64, WaveType::Triangle);
    // let mut oscillator = WaveTableOscillator::new(44100, wave_table);
    // oscillator.set_frequency(440.0);

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let _result = stream_handle.play_raw(oscillator.convert_samples());
    // std::thread::sleep(Duration::from_secs(1));
    let audio_state = AudioState::new(SpectrumType::Frequency);
    pollster::block_on(run(audio_state));
}
