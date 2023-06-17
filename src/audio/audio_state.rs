use crate::audio::util::{compute_sice_size, get_file, get_table_freq, get_table_time};

pub enum SpectrumType {
    Time,
    Frequency,
}

pub struct AudioState {
    pub spectrum_type: SpectrumType,
    pub samples: Vec<f32>,
    _size: usize,
    _sample_rate: u32,
    pub max_amplitude: f32,
    pub slice_size: usize,
}

impl AudioState {
    pub fn new(spectrum_type: SpectrumType) -> Self {
        // let file = FileInfo::new();

        let (samples, sample_rate) = get_file();
        let size = samples.len();
        let slice_size = compute_sice_size(sample_rate as f32, 60.0);

        let (samples, max_amplitude) = match spectrum_type {
            SpectrumType::Frequency => get_table_freq(samples, slice_size),
            SpectrumType::Time => get_table_time(&samples),
        };

        Self {
            spectrum_type,
            samples,
            _size: size,
            _sample_rate: sample_rate,
            max_amplitude,
            slice_size,
        }
    }
}
