use crate::audio::util::get_max_amplitude_freq;

#[derive(Clone)]
pub enum SpectrumType {
    Time,
    Frequency,
}

#[derive(Clone)]
pub struct AudioState {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub index: usize,
    pub spectrum_type: SpectrumType,
    pub size: usize,
    pub max_amplitude: f32,
    pub slice_size: usize,
}

pub struct AudioStateMetatada {
    pub spectrum_type: SpectrumType,
    pub size: usize,
    pub slice_size: usize,
    pub max_amplitude: f32,
}

impl AudioStateMetatada {
    fn new(samples: &Vec<f32>, sample_rate: f32) -> Self {
        let slice_size = compute_sice_size(sample_rate, 60.0);
        let max_amplitude = get_max_amplitude_freq(&samples, slice_size);

        Self {
            spectrum_type: SpectrumType::Frequency,
            size: samples.len(),
            slice_size,
            max_amplitude,
        }
    }
}

pub fn compute_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return (sample_rate / frame_rate) as usize;
}

impl AudioState {
    pub fn new(spectrum_type: SpectrumType, samples: Vec<f32>, sample_rate: u32) -> Self {
        let slice_size = compute_sice_size(sample_rate as f32, 60.0);

        let max_amplitude = match spectrum_type {
            SpectrumType::Frequency => get_max_amplitude_freq(&samples, slice_size),
            SpectrumType::Time => 1.0,
        };

        let size = samples.len();

        println!("max_amplitude: {}", max_amplitude);
        Self {
            spectrum_type,
            samples,
            size: size,
            sample_rate,
            max_amplitude,
            index: 0,
            slice_size,
        }
    }

    pub fn get_sample(&mut self) -> Option<f32> {
        self.index += 1;
        if self.index >= self.size {
            None
        } else {
            Some(self.samples[self.index])
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
