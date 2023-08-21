use dasp::Frame;

use super::audio_clip::{AudioClip, AudioClipTrait};

pub type EffectFn<F> = Box<dyn Fn(&mut AudioClip<F>) + Send + Sync>;

pub struct AudioEffectChain<F> {
    effects: Vec<EffectFn<F>>,
}

pub fn gain<F>(factor: f32) -> EffectFn<F>
where
    F: Frame<Sample = f32> + Send + Sync,
{
    Box::new(move |clip: &mut AudioClip<F>| {
        for frame in clip.get_mut_frames() {
            frame.mul_amp(factor);
        }
    })
}

pub fn invert<F>() -> EffectFn<F>
where
    F: Frame<Sample = f32> + Send + Sync,
{
    Box::new(|clip: &mut AudioClip<F>| {
        for frame in clip.get_mut_frames() {
            frame.scale_amp(-1.0);
        }
    })
}

pub fn fade_in<F>(duration: usize) -> EffectFn<F>
where
    F: Frame<Sample = f32> + Send + Sync,
{
    Box::new(move |clip: &mut AudioClip<F>| {
        for (i, frame) in clip.get_mut_frames().iter().enumerate() {
            let fade_factor = (i as f32) / (duration as f32);
            frame.scale_amp(fade_factor.min(1.0));
        }
    })
}

impl<F> AudioEffectChain<F>
where
    F: Frame<Sample = f32> + Send + Sync,
{
    pub fn new() -> Self {
        AudioEffectChain {
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: EffectFn<F>) {
        self.effects.push(effect);
    }

    pub fn apply(&self, clip: &mut AudioClip<F>) {
        for effect in &self.effects {
            effect(clip);
        }
    }
}

pub struct AudioNode<F> {
    pub name: String,
    clip: AudioClip<F>,
    effect_chain: Option<AudioEffectChain<F>>,
}

impl<F> AudioNode<F>
where
    F: dasp::Frame<Sample = f32> + Copy,
{
    pub fn new(clip: AudioClip<F>, name: Option<&str>) -> Self {
        let name = name.unwrap_or("default").to_string();
        AudioNode {
            name,
            clip,
            effect_chain: None,
        }
    }

    pub fn with_effects(
        clip: AudioClip<F>,
        effect_chain: AudioEffectChain<F>,
        name: Option<&str>,
    ) -> Self {
        let name = name.unwrap_or("default").to_string();
        let mut audio_node = AudioNode {
            name,
            clip,
            effect_chain: Some(effect_chain),
        };
        audio_node.process();
        audio_node
    }

    pub fn process(&mut self) {
        if let Some(effect_chain) = &self.effect_chain {
            effect_chain.apply(&mut self.clip);
        }
    }

    pub fn get_clip_ref(&self) -> &AudioClip<F> {
        &self.clip
    }

    pub fn get_clip_mut(&mut self) -> &mut AudioClip<F> {
        &mut self.clip
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dasp::frame::{Frame, Mono, Stereo};

    // Dummy AudioClip data for testing purposes
    fn get_mono_clip() -> AudioClip<Mono<f32>> {
        AudioClip {
            frames: vec![Mono::<f32>(0.5); 100], // 100 frames with value 0.5
        }
    }

    fn get_stereo_clip() -> AudioClip<Stereo<f32>> {
        AudioClip {
            frames: vec![Stereo::<f32>(0.5, 0.5); 100], // 100 frames with values 0.5 for both channels
        }
    }

    #[test]
    fn test_mono_effect_chain() {
        let mut clip = get_mono_clip();
        let mut effect_chain = AudioEffectChain::<Mono<f32>>::new();

        effect_chain.add_effect(gain(2.0)); // Expected to double the amplitude
        effect_chain.add_effect(invert()); // Expected to negate the sample value

        effect_chain.apply(&mut clip);

        for frame in clip.get_mut_frames() {
            assert_eq!(frame, &Mono::<f32>(-1.0)); // Check if the value is what we expect after applying the effects
        }
    }

    #[test]
    fn test_stereo_effect_chain() {
        let mut clip = get_stereo_clip();
        let mut effect_chain = AudioEffectChain::<Stereo<f32>>::new();

        effect_chain.add_effect(gain(2.0)); // Expected to double the amplitude
        effect_chain.add_effect(invert()); // Expected to negate the sample values

        effect_chain.apply(&mut clip);

        for frame in clip.get_mut_frames() {
            assert_eq!(frame, &Stereo::<f32>(-1.0, -1.0)); // Check if the values are what we expect after applying the effects
        }
    }

    #[test]
    fn test_mono_audio_node() {
        let clip = get_mono_clip();
        let mut node = AudioNode::new(clip, Some("test_mono_node"));

        assert_eq!(node.get_name(), "test_mono_node");

        let frame = node.get_clip_ref().get_frame(0).unwrap();
        assert_eq!(frame, &Mono::<f32>(0.5));
    }

    #[test]
    fn test_stereo_audio_node() {
        let clip = get_stereo_clip();
        let mut node = AudioNode::new(clip, Some("test_stereo_node"));

        assert_eq!(node.get_name(), "test_stereo_node");

        let frame = node.get_clip_ref().get_frame(0).unwrap();
        assert_eq!(frame, &Stereo::<f32>(0.5, 0.5));
    }
}
