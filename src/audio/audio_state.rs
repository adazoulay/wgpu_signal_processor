use dasp::Frame;
use dasp::slice;
use crate::audio::audio_clip::{AudioClip, AudioClipEnum, AudioClipTrait};
use crate::audio::util::get_max_amplitude_freq;
use dasp::frame::{Mono, Stereo};


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


impl AudioStateMetatada {
    pub fn new(sample_rate: f32) -> Self {
        let slice_size = compute_sice_size(sample_rate, 60.0);
        // let max_amplitude = get_max_amplitude_freq(&samples, slice_size);
        let max_amplitude = 1.0;
        let size = 0;

        Self {
            spectrum_type: SpectrumType::Time,
            size,
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

    pub fn update_metadata(&mut self, new_size: usize) {
        self.size = new_size;
    }

}


#[derive(Clone, Debug)]
pub struct AudioState<F> 
where
    F: Frame<Sample = f32> + Copy,
{
    pub samples: Vec<F>,
    pub sample_rate: u32,
    pub index: usize,
    pub audio_clips: Vec<F>,
    pub audio_metadata: AudioStateMetatada,
}


impl<F> AudioState<F> 
where
    F: Frame<Sample = f32> + Copy,
{
    pub fn new(sample_rate: u32) -> Self {
        let samples = Vec::new();

        let audio_metadata = AudioStateMetatada::new(sample_rate as f32);
        Self {
            samples,
            sample_rate,
            index: 0,
            audio_clips: Vec::new(),
            audio_metadata,
        }
    }
   
    pub fn get_sample(&mut self) -> Option<F> {
        self.index += 1;
        if self.index >= self.samples.len() {
            None
        } else {
            Some(self.samples[self.index])
        }
    }
    
    pub fn set_metadata(&mut self) {
        self.audio_metadata.update_metadata(self.samples.len());
    }

    pub fn get_metadata(&self) -> AudioStateMetatada {
        self.audio_metadata.clone()
    }
}

impl AudioState<[f32;1]> {
    pub fn add_clip(&mut self, mut clip: AudioClip<Mono<f32>>) {

        if self.sample_rate != clip.get_sample_rate() {
            clip = clip.resample(self.sample_rate);
        }
    
        let clip_samples = clip.get_samples();
        let min_len = std::cmp::min(self.samples.len(), clip_samples.len());
    
        for i in 0..min_len {
            self.samples[i] = (self.samples[i].scale_amp(0.5).add_amp(clip_samples[i].scale_amp(0.5))).into();
        }
    
        if clip_samples.len() > self.samples.len() {
            self.samples.extend_from_slice(&clip_samples[min_len..]);
        }
    }
}

impl AudioState<[f32;2]> {
    pub fn add_clip<F>(&mut self, mut clip: AudioClip<Stereo<f32>>) {
        if self.sample_rate != clip.get_sample_rate() {
            clip = clip.resample(self.sample_rate);
        }
    
        let clip_samples = clip.get_samples();
        let min_len = std::cmp::min(self.samples.len(), clip_samples.len());
    
        for i in 0..min_len {
            self.samples[i] = (self.samples[i].scale_amp(0.5).add_amp(clip_samples[i].scale_amp(0.5))).into();
        }
    
        if clip_samples.len() > self.samples.len() {
            self.samples.extend_from_slice(&clip_samples[min_len..]);
        }
    }
}


pub fn compute_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return (sample_rate / frame_rate) as usize;
}
