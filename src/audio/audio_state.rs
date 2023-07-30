use crate::audio::util::get_max_amplitude_freq;

#[derive(Clone, Debug)]
pub enum SpectrumType {
    Time,
    Frequency,
}

#[derive(Clone, Debug)]
pub struct AudioStateMetatada {
    pub spectrum_type: SpectrumType,
    pub size: usize,
    pub slice_size: usize,
    pub max_amplitude: f32,
}

#[derive(Clone, Debug)]
pub struct AudioState {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub index: usize,
    audio_metadata: AudioStateMetatada,
}

impl AudioStateMetatada {
    pub fn new(samples: &Vec<f32>, sample_rate: f32) -> Self {
        let slice_size = compute_sice_size(sample_rate, 60.0);
        let max_amplitude = get_max_amplitude_freq(&samples, slice_size);

        Self {
            spectrum_type: SpectrumType::Frequency,
            size: samples.len(),
            slice_size,
            max_amplitude,
        }
    }

    pub fn get_max_amplitude(&self) -> f32 {
        match self.spectrum_type {
            SpectrumType::Time => 1.0,
            SpectrumType::Frequency => self.max_amplitude,
        }
    }
}

impl AudioState {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let audio_metadata = AudioStateMetatada::new(&samples, sample_rate as f32);

        Self {
            samples,
            sample_rate,
            index: 0,
            audio_metadata,
        }
    }

    pub fn get_sample(&mut self) -> Option<f32> {
        self.index += 1;
        if self.index >= self.samples.len() {
            None
        } else {
            Some(self.samples[self.index])
        }
    }

    pub fn get_metadata(&self) -> AudioStateMetatada {
        self.audio_metadata.clone()
    }
}

pub fn compute_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return 8 * (sample_rate * 4.0 / frame_rate) as usize;
}

impl Iterator for AudioState {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_sample()
    }
}
