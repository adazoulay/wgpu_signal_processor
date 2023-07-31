use dasp::frame::{Stereo, Mono};
use dasp::frame::Frame;
use dasp::{ interpolate::linear::Linear, Signal};
use dasp::signal;


#[derive(Clone, Debug)]
pub enum AudioClipEnum {
    Mono(AudioClip<Mono<f32>>),
    Stereo(AudioClip<Stereo<f32>>),
}


// impl AudioClipTrait for AudioClipEnum {
//     type S = f32; // or whatever your S type should be

//     fn get_samples(&self) -> &[Self::S] {
//         match self {
//             AudioClipEnum::Mono(clip) => clip.get_samples(),
//             AudioClipEnum::Stereo(clip) => clip.get_samples(),
//         }
//     }

//     // ...implement the other methods similarly
// }

pub trait AudioClipTrait {
    type S: dasp::Frame;

    fn get_samples(&self) -> &[Self::S];

    fn get_sample(&self, idx: usize) -> Self::S ;

    fn get_sample_rate(&self) -> u32;

    fn resample(&self, sample_rate: u32) -> Self
    where
        Self: Sized;

}

#[derive(Clone, Debug)]
pub struct AudioClip<F> {
    samples: Vec<F>,
    sample_rate: u32,
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

    fn resample(&self, sample_rate: u32) -> Self {
        if self.sample_rate == sample_rate {
            return self.clone();
        }

        let mut signal = signal::from_iter(self.samples.iter().copied());
        let a = signal.next();
        let b = signal.next();
        let linear = Linear::new(a, b);

        Self {
            samples: signal
                .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
                .take(self.samples.len() * (sample_rate as usize) / self.sample_rate as usize)
                .collect(),
            sample_rate,
        }
    }    
}

impl AudioClip<[f32; 1]> {

    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let samples: Vec<[f32; 1]> = samples.into_iter().map(|sample| [sample]).collect();
        Self {
            samples,
            sample_rate
        }
    }

    pub fn to_stereo(&self) -> AudioClip<[f32; 2]> { // [f32; 2]: Stereo
        let stereo_samples = self.samples.iter().map(|mono| [mono[0],mono[0]]).collect();
        AudioClip {
            samples: stereo_samples,
            sample_rate: self.sample_rate,
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
            samples: frames,
            sample_rate,
        }
    }
    
    

    pub fn to_mono(&self) -> AudioClip<[f32; 1]> {
        let mono_samples = self.samples.iter().map(|stereo| [(stereo[0] + stereo[1]) / 2.0]).collect();
        AudioClip {
            samples: mono_samples,
            sample_rate: self.sample_rate,
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
        };
        let sample = input_clip.get_sample(2);
        assert_eq!(sample, [2.0, 2.0]);
    }
}
