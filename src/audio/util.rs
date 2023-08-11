use rustfft::{num_complex::Complex, FftPlanner};


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

use std::env;
use std::fs::File;
use std::path::PathBuf;


pub fn from_file() -> Option<(Vec<f32>, u32, u32)> {
    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/audio/");
    path.push("test.flac");

    let file = File::open(&path).unwrap();
    let mut reader = audrey::Reader::new(file).unwrap();
    let desc = reader.description();
    let sample_rate = desc.sample_rate() as u32;
    let channels = desc.channel_count();

    let samples: Vec<f32> = reader.samples::<f32>().filter_map(Result::ok).collect();

    Some((samples, sample_rate, channels))
}
