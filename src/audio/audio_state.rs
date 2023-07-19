use crate::audio::util::{get_table_freq, get_table_time};

#[derive(Clone)]
pub enum SpectrumType {
    Time,
    Frequency,
}

#[derive(Clone)]
pub struct AudioState {
    pub spectrum_type: SpectrumType,
    pub samples: Vec<f32>,
    pub size: usize,
    pub sample_rate: u32,
    pub max_amplitude: f32,
    pub index: usize,
    pub slice_size: usize,
    pub slice_index: usize,
}

pub fn compute_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return (sample_rate / frame_rate) as usize;
}

impl AudioState {
    pub fn new(spectrum_type: SpectrumType, samples: Vec<f32>, sample_rate: u32) -> Self {
        let slice_size = compute_sice_size(sample_rate as f32, 60.0);

        let (samples, max_amplitude) = match spectrum_type {
            SpectrumType::Frequency => get_table_freq(samples, slice_size),
            SpectrumType::Time => get_table_time(&samples),
        };

        let size = samples.len();

        Self {
            spectrum_type,
            samples,
            size: size,
            sample_rate,
            max_amplitude,
            index: 0,
            slice_size,
            slice_index: 0,
        }
    }

    pub fn get_slice(&mut self) -> Option<&[f32]> {
        let slice = &self.samples
            [self.slice_index * self.slice_size..(self.slice_index + 1) * self.slice_size];
        Some(slice)
    }

    pub fn get_next_slice(&mut self) -> Option<&[f32]> {
        if self.slice_index * self.slice_size >= self.size {
            return None;
        }
        let slice = &self.samples
            [self.slice_index * self.slice_size..(self.slice_index + 1) * self.slice_size];
        self.slice_index += 1;
        self.index = 0;
        Some(slice)
    }

    pub fn get_sample(&mut self) -> Option<f32> {
        if self.index >= self.slice_size {
            if self.get_next_slice().is_none() {
                return None;
            }
        }

        let sample_index = self.slice_index * self.slice_size + self.index;
        self.index += 1;

        if sample_index >= self.size {
            None
        } else {
            Some(self.samples[sample_index])
        }
    }
}

impl Iterator for AudioState {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_sample()
    }
}

use rodio::Source;

impl Source for AudioState {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        return None;
    }
}
