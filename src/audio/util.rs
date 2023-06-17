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
    path.push("src");
    path.push("audio");
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
            let f = sample as f32;
            samples.push(f);
        }
    }

    (samples, sample_rate as u32)
}

pub fn compute_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return (sample_rate / frame_rate) as usize;
}

pub fn get_table_time(audio_samples: &[f32]) -> (Vec<f32>, f32) {
    let mut max_amplitude = 0.0_f32;

    let result = audio_samples
        .iter()
        .map(|&sample| {
            if sample.abs() > max_amplitude {
                max_amplitude = sample.abs();
            }
            sample
        })
        .collect();

    (result, max_amplitude)
}

pub fn get_table_freq(samples: Vec<f32>, slice_size: usize) -> (Vec<f32>, f32) {
    let fft = FftPlanner::new().plan_fft_forward(slice_size);
    let num_slices = samples.len() / slice_size;

    let mut table = Vec::with_capacity(num_slices * slice_size);
    let mut max_amplitude = 0.0;

    // Define Hanning window
    let window: Vec<f32> = (0..slice_size)
        .map(|i| {
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (slice_size - 1) as f32).cos())
        })
        .collect();

    for slice_idx in 0..num_slices {
        let start = slice_idx * slice_size;
        let end = start + slice_size;
        let slice = &samples[start..end];

        // Apply FFT to the slice
        let mut fft_input: Vec<Complex<f32>> = slice
            .iter()
            .enumerate()
            .map(|(i, x)| Complex::new(*x * window[i], 0.0))
            .collect();
        fft.process(&mut fft_input);

        // Normalize the FFT results and convert the data
        for x in fft_input {
            let y_component = (x.norm() / slice_size as f32).sqrt(); // sqrt for perceptual scaling, divided by slice_size to normalize FFT output
            if y_component > max_amplitude {
                max_amplitude = y_component;
            }
            table.push(y_component);
        }
    }

    (table, max_amplitude)
}

pub fn hanning_window(length: usize) -> Vec<f32> {
    (0..length)
        .map(|i| {
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (length as f32 - 1.0)).cos())
        })
        .collect()
}
