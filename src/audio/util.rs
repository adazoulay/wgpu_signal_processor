// let mut audio_data = process_audio_data(&raw_audio_data);
// normalize_coordinates(&mut audio_data);
use minimp3::{Decoder as Mp3Decoder, Frame};
use rustfft::{num_complex::Complex, FftPlanner};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

// ! FFT AND NORMALIZATION
pub fn raw_to_fft(audio_data: &[f32], sample_rate: usize) -> Vec<[f32; 2]> {
    let mut buffer: Vec<Complex<f32>> = audio_data.iter().map(|x| Complex::new(*x, 0.0)).collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer.len());
    fft.process(&mut buffer);

    let normalization_factor = (buffer.len() as f32).sqrt();
    let half_len = buffer.len() / 2;
    let freq_increment = sample_rate as f32 / buffer.len() as f32;

    buffer
        .iter()
        .take(half_len)
        .enumerate()
        .map(|(i, x)| {
            let frequency = i as f32 * freq_increment;
            let magnitude = x.norm() / normalization_factor;
            [frequency, magnitude]
        })
        .collect()
}

pub fn normalize_coordinates(audio_data: &mut Vec<[f32; 2]>) {
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);

    for point in audio_data.iter() {
        min_x = min_x.min(point[0]);
        max_x = max_x.max(point[0]);
        min_y = min_y.min(point[1]);
        max_y = max_y.max(point[1]);
    }

    let range_x = max_x - min_x;
    let range_y = max_y - min_y;

    for point in audio_data.iter_mut() {
        point[0] = (point[0] - min_x) / range_x * 2.0 - 1.0;
        point[1] = (point[1] - min_y) / range_y * 2.0 - 1.0;
    }
}

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

pub fn compute_fft(samples: &[f32], slice_size: usize) -> (Vec<[f32; 2]>, f32) {
    let fft = FftPlanner::new().plan_fft_forward(slice_size);
    let num_slices = samples.len() / slice_size;

    let mut max_amplitude = 0.0_f32;
    let mut result = Vec::with_capacity(num_slices * slice_size);

    for slice_idx in 0..num_slices {
        let start = slice_idx * slice_size;
        let end = start + slice_size;
        let slice = &samples[start..end];

        let mut fft_input: Vec<Complex<f32>> =
            slice.iter().map(|x| Complex::new(*x, 0.0)).collect();
        fft.process(&mut fft_input);

        for complex in fft_input.iter() {
            let amplitude = complex.norm();
            if amplitude > max_amplitude {
                max_amplitude = amplitude;
            }
            result.push([complex.re, complex.im]);
        }
    }

    (result, max_amplitude)
}

pub fn compute_time_domain(audio_samples: &[f32], sample_rate: u32) -> (Vec<[f32; 2]>, f32) {
    let time_increment = 1.0 / sample_rate as f32;

    let mut max_amplitude = 0.0_f32;

    let result = audio_samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let time = i as f32 * time_increment;
            if sample.abs() > max_amplitude {
                max_amplitude = sample.abs();
            }
            [time, sample]
        })
        .collect();

    (result, max_amplitude)
}
