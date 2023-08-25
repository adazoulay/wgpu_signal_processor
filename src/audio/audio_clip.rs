use dasp::frame::Frame;
use dasp::frame::{Mono, Stereo};
use dasp::signal;
use dasp::{interpolate::linear::Linear, Signal};

pub trait AudioClipTrait {
    type S: dasp::Frame;

    // Constructors
    fn default() -> Self;
    fn with_capacity(capacity: usize) -> Self;

    // Getters
    fn get_frames_ref(&self) -> &[Self::S];
    fn get_frames_mut(&mut self) -> &mut [Self::S];
    fn get_frame(&self, idx: usize) -> Option<Self::S>;
    fn get_sample_rate(&self) -> u32;
    fn get_start_time_frame(&self) -> usize;
    fn get_length(&self) -> usize;

    // Setters
    fn set_frame(&mut self, idx: usize, val: Self::S);
    fn set_start_time_frame(&mut self, sample_idx: usize);
    fn resample(&self, sample_rate: u32) -> Self
    where
        Self: Sized;
    fn resize_frames(&mut self, new_size: usize, value: Self::S);
    fn add_padding_left(&mut self, padding_frames: usize);
    fn reset_clip(&mut self);
}

#[derive(Clone, Debug)]
pub struct AudioClip<F> {
    frames: Vec<F>,
    sample_rate: u32,
    start_time_frame: usize,
}

impl<F> AudioClipTrait for AudioClip<F>
where
    F: Frame<Sample = f32> + Copy,
{
    type S = F;

    // Initializer
    fn default() -> Self {
        let sample_rate = 44100;
        let length = sample_rate * 5;
        let frames = vec![F::EQUILIBRIUM; length];

        Self {
            frames,
            sample_rate: sample_rate as u32,
            start_time_frame: 0,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        let sample_rate = 44100;
        let frames = vec![F::EQUILIBRIUM; capacity];

        Self {
            frames,
            sample_rate: sample_rate as u32,
            start_time_frame: 0,
        }
    }

    // Getters
    fn get_frames_ref(&self) -> &[Self::S] {
        &self.frames
    }

    fn get_frames_mut(&mut self) -> &mut [Self::S] {
        &mut self.frames
    }

    fn get_frame(&self, idx: usize) -> Option<Self::S> {
        if idx < self.get_length() {
            Some(self.frames[idx])
        } else {
            None
        }
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn get_start_time_frame(&self) -> usize {
        self.start_time_frame
    }

    fn get_length(&self) -> usize {
        self.frames.len()
    }

    // Setters
    // Todo Should return Results
    fn set_frame(&mut self, idx: usize, val: Self::S) {
        self.frames[idx] = val;
    }

    fn set_start_time_frame(&mut self, sample_idx: usize) {
        self.start_time_frame = sample_idx;
    }

    fn resize_frames(&mut self, new_size: usize, value: Self::S) {
        self.frames.resize(new_size, value);
    }

    fn add_padding_left(&mut self, padding_frames: usize) {
        if padding_frames <= 0 {
            return;
        }
        let mut padding = vec![F::EQUILIBRIUM; padding_frames];
        padding.append(&mut self.frames);
        self.frames = padding;
    }

    fn resample(&self, sample_rate: u32) -> Self {
        if self.sample_rate == sample_rate {
            return self.clone();
        }

        let mut signal = signal::from_iter(self.frames.iter().copied());
        let a = signal.next();
        let b = signal.next();
        let linear = Linear::new(a, b);
        let frames: Vec<F> = signal
            .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
            .take(self.frames.len() * (sample_rate as usize) / self.sample_rate as usize)
            .collect();

        Self {
            frames,
            sample_rate,
            start_time_frame: self.start_time_frame,
        }
    }

    fn reset_clip(&mut self) {
        let length = self.get_length();
        let frames = vec![F::EQUILIBRIUM; length];
        self.frames = frames;
    }
}

impl AudioClip<[f32; 1]> {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let frames: Vec<[f32; 1]> = samples.into_iter().map(|sample| [sample]).collect();
        Self {
            frames,
            sample_rate,
            start_time_frame: 0,
        }
    }

    pub fn to_stereo(&self) -> AudioClip<[f32; 2]> {
        let stereo_frames: Vec<[f32; 2]> =
            self.frames.iter().map(|mono| [mono[0], mono[0]]).collect();
        AudioClip {
            frames: stereo_frames,
            sample_rate: self.sample_rate,
            start_time_frame: 0,
        }
    }
}

impl AudioClip<[f32; 2]> {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let frames: Vec<[f32; 2]> = samples
            .chunks_exact(2)
            .map(|chunk| [chunk[0], chunk[1]])
            .collect();

        Self {
            frames,
            sample_rate,
            start_time_frame: 0,
        }
    }

    pub fn to_mono(&self) -> AudioClip<[f32; 1]> {
        let mono_frames: Vec<[f32; 1]> = self
            .frames
            .iter()
            .map(|stereo| [(stereo[0] + stereo[1]) / 2.0])
            .collect();

        AudioClip {
            frames: mono_frames,
            sample_rate: self.sample_rate,
            start_time_frame: 0,
        }
    }
}

pub enum AudioClipEnum {
    Mono(AudioClip<Mono<f32>>),
    Stereo(AudioClip<Stereo<f32>>),
}

impl AudioClipEnum {
    pub fn from_samples(samples: Vec<f32>, sample_rate: u32, channels: u32) -> Self {
        match channels {
            1 => Self::Mono(AudioClip::<Mono<f32>>::new(samples, sample_rate)),
            2 => Self::Stereo(AudioClip::<Stereo<f32>>::new(samples, sample_rate)),
            _ => panic!("Invalid number of channels"),
        }
    }

    pub fn default() -> Self {
        let sample_rate = 44100;
        let samples = Vec::<f32>::with_capacity(sample_rate * 5);
        let clip = AudioClip::<Stereo<f32>>::new(samples, sample_rate as u32);
        AudioClipEnum::Stereo(clip)
    }
}

// ! ---------  Tests ---------

#[cfg(test)]
mod tests {
    use super::*;
    use dasp::frame::{Mono, Stereo};

    #[test]
    fn test_resample_mono() {
        let input_samples: Vec<Mono<f32>> = vec![[0.0]; 1000];
        let input_clip = AudioClip {
            frames: input_samples,
            sample_rate: 44100,
            start_time_frame: 0,
        };

        let output_clip = input_clip.resample(88200);
        assert_eq!(output_clip.frames.len(), 2000);
        assert_eq!(output_clip.sample_rate, 88200);
    }

    #[test]
    fn test_resample_stereo() {
        let input_samples: Vec<Stereo<f32>> = vec![[0.0, 0.0]; 1000];
        let input_clip = AudioClip {
            frames: input_samples,
            sample_rate: 44100,
            start_time_frame: 0,
        };

        let output_clip = input_clip.resample(88200);
        assert_eq!(output_clip.frames.len(), 2000);
        assert_eq!(output_clip.sample_rate, 88200);
    }

    #[test]
    fn test_get_samples_mono() {
        let input_samples: Vec<Mono<f32>> = vec![[0.0; 1]; 1000];
        let input_clip = AudioClip {
            frames: input_samples,
            sample_rate: 44100,
            start_time_frame: 0,
        };
        let samples = input_clip.get_frames_ref();
        assert_eq!(samples, vec![[0.0; 1]; 1000]);
    }

    #[test]
    fn test_get_samples_stereo() {
        let input_samples: Vec<Stereo<f32>> = vec![[0.0, 0.0]; 1000];
        let input_clip = AudioClip {
            frames: input_samples,
            sample_rate: 44100,
            start_time_frame: 0,
        };
        let samples = input_clip.get_frames_ref();
        assert_eq!(samples, vec![[0.0, 0.0]; 1000]);
    }

    #[test]
    fn test_get_sample_mono() {
        let input_samples: Vec<Mono<f32>> = vec![[0.0], [1.0], [2.0], [3.0], [4.0]];
        let input_clip = AudioClip {
            frames: input_samples,
            sample_rate: 44100,
            start_time_frame: 0,
        };
        let sample = input_clip.get_frame(2).unwrap();
        assert_eq!(sample, [2.0]);
    }

    #[test]
    fn test_get_sample_stereo() {
        let input_samples: Vec<Stereo<f32>> =
            vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
        let input_clip = AudioClip {
            frames: input_samples,
            sample_rate: 44100,
            start_time_frame: 0,
        };
        let sample = input_clip.get_frame(2).unwrap();
        assert_eq!(sample, [2.0, 2.0]);
    }
}
