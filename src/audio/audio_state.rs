use crate::audio::audio_clip::{AudioClip, AudioClipEnum, AudioClipTrait};
use dasp::Frame;

#[derive(Clone, Debug)]
pub struct AudioState<F>
where
    F: Frame<Sample = f32> + Copy,
{
    pub samples: Vec<F>,
    pub sample_rate: u32,
    pub index: usize,
    pub audio_clips: Vec<AudioClip<F>>,
    pub audio_metadata: AudioStateMetatada,
}

impl<F> AudioState<F>
where
    F: Frame<Sample = f32> + Copy,
{
    pub fn new(sample_rate: u32) -> Self {
        let audio_metadata = AudioStateMetatada::new(sample_rate as f32);

        Self {
            samples: Vec::new(),
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

    pub fn add_clip_to_samples(&mut self, clip: AudioClip<F>) {
        let s = clip.get_start_time_frame() as usize;
        let l = clip.get_length() as usize;

        if s + l > self.samples.len() {
            let p = s + l;
            self.samples.resize(p, F::EQUILIBRIUM);
        }

        let clip_samples = clip.get_frames();
        for i in s..(s + clip.get_length() as usize) {
            self.samples[i] = (self.samples[i].add_amp(clip_samples[i - s])).into();
        }

        self.audio_clips.push(clip);
        self.update_metadata();
    }

    pub fn update_metadata(&mut self) {
        self.audio_metadata.update_metadata(self.samples.len());
    }

    pub fn get_metadata(&self) -> AudioStateMetatada {
        self.audio_metadata.clone()
    }
}

impl AudioState<[f32; 1]> {
    pub fn add_clip(&mut self, clip: AudioClipEnum) {
        let mut clip = match clip {
            AudioClipEnum::Mono(clip) => clip,
            AudioClipEnum::Stereo(clip) => clip.to_mono(),
        };

        if self.sample_rate != clip.get_sample_rate() {
            clip = clip.resample(self.sample_rate);
        }

        self.add_clip_to_samples(clip);
    }
}

impl AudioState<[f32; 2]> {
    pub fn add_clip<F>(&mut self, clip: AudioClipEnum) {
        let mut clip = match clip {
            AudioClipEnum::Mono(clip) => clip.to_stereo(),
            AudioClipEnum::Stereo(clip) => clip,
        };

        if self.sample_rate != clip.get_sample_rate() {
            clip = clip.resample(self.sample_rate);
        }

        self.add_clip_to_samples(clip);
    }
}

// Metadata and Type

pub fn compute_sice_size(sample_rate: f32, frame_rate: f32) -> usize {
    return (sample_rate / frame_rate) as usize;
}

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
        let slice_size = compute_sice_size(sample_rate, 30.0);
        let max_amplitude = 0.3;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_clip_to_samples_mono_start_2() {
        let sample_rate = 44100;
        let mut audio_state = AudioState::<[f32; 1]>::new(sample_rate);
        audio_state.samples = vec![[0.0], [1.0], [2.0], [3.0], [4.0]];
        let mut clip = AudioClip::<[f32; 1]>::new(vec![99.0, 99.0, 99.0, 99.0], sample_rate);
        clip.set_start_time_frame(2);
        audio_state.add_clip_to_samples(clip);
        assert_eq!(
            audio_state.samples,
            vec![[0.0], [1.0], [101.0], [102.0], [103.0], [99.0]]
        );
    }

    #[test]
    fn test_add_clip_to_samples_mono_start_0() {
        let sample_rate = 44100;
        let mut audio_state = AudioState::<[f32; 1]>::new(sample_rate);
        audio_state.samples = vec![[0.0], [1.0], [2.0], [3.0], [4.0]];
        let mut clip = AudioClip::<[f32; 1]>::new(vec![99.0, 99.0, 99.0, 99.0], sample_rate);
        clip.set_start_time_frame(0);
        audio_state.add_clip_to_samples(clip);
        assert_eq!(
            audio_state.samples,
            vec![[99.0], [100.0], [101.0], [102.0], [4.0]]
        );
    }

    #[test]
    fn test_add_clip_to_samples_mono_start_9() {
        let sample_rate = 44100;
        let mut audio_state = AudioState::<[f32; 1]>::new(sample_rate);
        audio_state.samples = vec![[0.0], [1.0], [2.0], [3.0], [4.0]];
        let mut clip = AudioClip::<[f32; 1]>::new(vec![99.0, 99.0, 99.0, 99.0], sample_rate);
        clip.set_start_time_frame(9);
        audio_state.add_clip_to_samples(clip);
        assert_eq!(
            audio_state.samples,
            vec![
                [0.0],
                [1.0],
                [2.0],
                [3.0],
                [4.0],
                [0.0],
                [0.0],
                [0.0],
                [0.0],
                [99.0],
                [99.0],
                [99.0],
                [99.0]
            ]
        );
    }

    #[test]
    fn test_add_clip_to_samples_stereo_start_2() {
        let sample_rate = 44100;
        let mut audio_state = AudioState::<[f32; 2]>::new(sample_rate);
        audio_state.samples = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
        let v: Vec<f32> = vec![99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0];
        let mut clip = AudioClip::<[f32; 2]>::new(v, sample_rate);
        clip.set_start_time_frame(2);
        audio_state.add_clip_to_samples(clip);
        assert_eq!(
            audio_state.samples,
            vec![
                [0.0, 0.0],
                [1.0, 1.0],
                [101.0, 101.0],
                [102.0, 102.0],
                [103.0, 103.0],
                [99.0, 99.0]
            ]
        );
    }

    #[test]
    fn test_add_clip_to_samples_stereo_start_0() {
        let sample_rate = 44100;
        let mut audio_state = AudioState::<[f32; 2]>::new(sample_rate);
        audio_state.samples = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
        let v: Vec<f32> = vec![99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0];
        let mut clip = AudioClip::<[f32; 2]>::new(v, sample_rate);
        clip.set_start_time_frame(0);
        audio_state.add_clip_to_samples(clip);
        assert_eq!(
            audio_state.samples,
            vec![
                [99.0, 99.0],
                [100.0, 100.0],
                [101.0, 101.0],
                [102.0, 102.0],
                [4.0, 4.0]
            ]
        );
    }

    #[test]
    fn test_add_clip_to_samples_stereo_start_9() {
        let sample_rate = 44100;
        let mut audio_state = AudioState::<[f32; 2]>::new(sample_rate);
        audio_state.samples = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
        let v: Vec<f32> = vec![99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0];
        let mut clip = AudioClip::<[f32; 2]>::new(v, sample_rate);
        clip.set_start_time_frame(9);
        audio_state.add_clip_to_samples(clip);
        assert_eq!(
            audio_state.samples,
            vec![
                [0.0, 0.0],
                [1.0, 1.0],
                [2.0, 2.0],
                [3.0, 3.0],
                [4.0, 4.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [99.0, 99.0],
                [99.0, 99.0],
                [99.0, 99.0],
                [99.0, 99.0]
            ]
        );
    }
}
