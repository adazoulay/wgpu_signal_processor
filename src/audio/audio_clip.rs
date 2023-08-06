use dasp::frame::{Stereo, Mono};
use dasp::frame::Frame;
use dasp::{ interpolate::linear::Linear, Signal};
use dasp::signal;


#[derive(Clone, Debug)]
pub enum AudioClipEnum {
    Mono(AudioClip<Mono<f32>>),
    Stereo(AudioClip<Stereo<f32>>),
}

impl AudioClipEnum {
    pub fn from_samples(samples: Vec<f32>, sample_rate: u32, channels: u32) -> Self {
        match channels {
            1 => Self::Stereo(AudioClip::<Mono<f32>>::new(samples, sample_rate).to_stereo()),
            2 => Self::Mono(AudioClip::<Stereo<f32>>::new(samples, sample_rate).to_mono()),
            _ => panic!("Invalid number of channels"),
        }
    }
}


pub trait AudioClipTrait {
    type S: dasp::Frame;

    fn get_samples(&self) -> &[Self::S];

    fn get_sample(&self, idx: usize) -> Self::S ;

    fn get_sample_rate(&self) -> u32;

    fn get_start_time_samples(&self) -> u32;

    fn set_start_time_samples(&mut self, sample_idx: u32);
    
    fn get_length(&self) -> u32;

    fn resample(&self, sample_rate: u32) -> Self
    where
        Self: Sized;
   
}

#[derive(Clone, Debug)]
pub struct AudioClip<F> {
    samples: Vec<F>,
    sample_rate: u32,
    length: u32,
    start_time_samples: u32,
}

impl<F> AudioClipTrait for AudioClip<F>
where
    F: Frame<Sample = f32> + Copy,
{
    type S = F;

    fn get_samples(&self) -> &[Self::S] {
        &self.samples
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn get_sample(&self, idx: usize) -> Self::S {
        self.samples[idx]
    }

    fn get_start_time_samples(&self) -> u32{
        self.start_time_samples
    }
    fn set_start_time_samples(&mut self, sample_idx: u32) {
        self.start_time_samples = sample_idx;
    }

    fn get_length(&self) -> u32 {
        self.length
    }

    fn resample(&self, sample_rate: u32) -> Self {
        if self.sample_rate == sample_rate {
            return self.clone();
        }

        let mut signal = signal::from_iter(self.samples.iter().copied());
        let a = signal.next();
        let b = signal.next();
        let linear = Linear::new(a, b);
        let samples: Vec<F> = signal
        .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
        .take(self.samples.len() * (sample_rate as usize) / self.sample_rate as usize)
        .collect();

        let length = samples.len() as u32;

        Self {
            samples,
            sample_rate,
            length,
            start_time_samples: 0,
        }
    }
    
}


impl AudioClip<[f32; 1]> {
    // ! New might be able to be abstracted to main impl
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let samples: Vec<[f32; 1]> = samples.into_iter().map(|sample| [sample]).collect();
        let length = samples.len() as u32;
        Self {
            samples,
            sample_rate,
            length,
            start_time_samples: 0,
        }
    }

    pub fn to_stereo(&self) -> AudioClip<[f32; 2]> { // [f32; 2]: Stereo
        let stereo_samples: Vec<[f32;2]>  = self.samples.iter().map(|mono| [mono[0],mono[0]]).collect();
        let length = stereo_samples.len() as u32;
        AudioClip {
            samples: stereo_samples,
            sample_rate: self.sample_rate,
            length,
            start_time_samples: 0,
        }
    }
}

impl AudioClip<[f32; 2]> {

    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let frames: Vec<[f32; 2]> = samples
            .chunks_exact(2)
            .map(|chunk| [chunk[0], chunk[1]])
            .collect();

        let length = frames.len() as u32;
    
        Self {
            samples: frames,
            sample_rate,
            length,
            start_time_samples: 0,
        }
    }
    
    pub fn to_mono(&self) -> AudioClip<[f32; 1]> {
        let mono_samples: Vec<[f32;1]>  = self.samples.iter().map(|stereo| [(stereo[0] + stereo[1]) / 2.0]).collect();

        let length = mono_samples.len() as u32;

        AudioClip {
            samples: mono_samples,
            sample_rate: self.sample_rate,
            length,
            start_time_samples: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use dasp::frame::{Mono, Stereo};

    #[test]
    fn test_resample_mono() {
        let input_samples: Vec<Mono<f32>> = vec![[0.0]; 1000];
        let input_clip = AudioClip {
            samples: input_samples,
            sample_rate: 44100,
            start_time_samples: 0,
            length: 0,
        };

        let output_clip = input_clip.resample(88200);
        assert_eq!(output_clip.samples.len(), 2000);
        assert_eq!(output_clip.sample_rate, 88200);
    }

    #[test]
    fn test_resample_stereo() {
        let input_samples: Vec<Stereo<f32>> = vec![[0.0, 0.0]; 1000];
        let input_clip = AudioClip {
            samples: input_samples,
            sample_rate: 44100,
            start_time_samples: 0,
            length: 0,
        };

        let output_clip = input_clip.resample(88200);
        assert_eq!(output_clip.samples.len(), 2000);
        assert_eq!(output_clip.sample_rate, 88200);
    }

    #[test]
    fn test_get_samples_mono() {
        let input_samples: Vec<Mono<f32>> = vec![[0.0; 1]; 1000];
        let input_clip = AudioClip {
            samples: input_samples,
            sample_rate: 44100,
            start_time_samples: 0,
            length: 0,
        };
        let samples = input_clip.get_samples();
        assert_eq!(samples, vec![[0.0; 1]; 1000]);
    }
    
    #[test]
    fn test_get_samples_stereo() {
        let input_samples: Vec<Stereo<f32>> = vec![[0.0, 0.0]; 1000];
        let input_clip = AudioClip {
            samples: input_samples,
            sample_rate: 44100,
            start_time_samples: 0,
            length: 0,
        };
        let samples = input_clip.get_samples();
        assert_eq!(samples, vec![[0.0, 0.0]; 1000]);
    }

    #[test]
    fn test_get_sample_mono() {
        let input_samples: Vec<Mono<f32>> = vec![[0.0] ,[1.0],[2.0],[3.0],[4.0]];
        let input_clip = AudioClip {
            samples: input_samples,
            sample_rate: 44100,
            start_time_samples: 0,
            length: 0,
        };
        let sample = input_clip.get_sample(2);
        assert_eq!(sample, [2.0]);
    }
    
    #[test]
    fn test_get_sample_stereo() {
        let input_samples: Vec<Stereo<f32>> = vec![[0.0, 0.0] ,[1.0, 1.0],[2.0, 2.0],[3.0, 3.0],[4.0, 4.0]];
        let input_clip = AudioClip {
            samples: input_samples,
            sample_rate: 44100,
            start_time_samples: 0,
            length: 0,
        };
        let sample = input_clip.get_sample(2);
        assert_eq!(sample, [2.0, 2.0]);
    }
}
