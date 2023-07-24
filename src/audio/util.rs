// let mut audio_data = process_audio_data(&raw_audio_data);
// normalize_coordinates(&mut audio_data);
use minimp3::{Decoder as Mp3Decoder, Frame};
use rustfft::{num_complex::Complex, FftPlanner};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn get_file() -> (Vec<f32>, u32) {
    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/audio/");
    path.push("test.mp3");

    let mut mp3_data = Vec::new();

    File::open(&path)
        .unwrap()
        .read_to_end(&mut mp3_data)
        .unwrap();

    let mut mp3_decoder = Mp3Decoder::new(mp3_data.as_slice());
    let mut samples = Vec::new();
    let mut sample_rate = 0;

    while let Ok(Frame {
        data,
        sample_rate: sr,
        ..
    }) = mp3_decoder.next_frame()
    {
        sample_rate = sr;
        for sample in data {
            let f = sample as f32 / 32768.0; // Normalize to -1.0 to 1.0
            samples.push(f);
        }
    }

    (samples, sample_rate as u32)
}

pub fn get_max_amplitude_freq(samples: &Vec<f32>, reduced_slice_size: usize) -> f32 {
    let fft = FftPlanner::new().plan_fft_forward(reduced_slice_size);
    let num_slices = samples.len() / reduced_slice_size;

    let mut max_amplitude = 0.0;

    for slice_idx in 0..num_slices {
        let start = slice_idx * reduced_slice_size;
        let end = start + reduced_slice_size;
        let slice = &samples[start..end];

        // Apply FFT to the slice
        let mut fft_input: Vec<Complex<f32>> =
            slice.iter().map(|&x| Complex::new(x, 0.0)).collect();
        fft.process(&mut fft_input);

        // Get the max amplitude
        for x in fft_input {
            let y_component = (x.norm() / reduced_slice_size as f32).sqrt();
            if y_component > max_amplitude {
                max_amplitude = y_component;
            }
        }
    }

    max_amplitude
}
