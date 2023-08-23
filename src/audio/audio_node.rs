use super::audio_clip::{AudioClip, AudioClipTrait};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct AudioNode<F> {
    pub name: String,
    clip: Arc<Mutex<AudioClip<F>>>,
    // effect_chain: Option<AudioEffectChain<F>>,
}

impl<F> AudioNode<F>
where
    F: dasp::Frame<Sample = f32> + Copy,
{
    pub fn new(clip: AudioClip<F>, name: Option<&str>) -> Self {
        let name = name.unwrap_or("default").to_string();
        AudioNode {
            name,
            clip: Arc::new(Mutex::new(clip)),
            // effect_chain: None,
        }
    }

    pub fn get_clip(&self) -> std::sync::MutexGuard<'_, AudioClip<F>> {
        self.clip.lock().unwrap()
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    // pub fn with_effects(
    //     clip: AudioClip<F>,
    //     // effect_chain: AudioEffectChain<F>,
    //     name: Option<&str>,
    // ) -> Self {
    //     let name = name.unwrap_or("default").to_string();
    //     let mut audio_node = AudioNode {
    //         name,
    //         clip,
    //         // effect_chain: Some(effect_chain),
    //     };
    //     audio_node.process();
    //     audio_node
    // }

    // pub fn process(&mut self) {
    //     if let Some(effect_chain) = &self.effect_chain {
    //         effect_chain.apply(&mut self.clip);
    //     }
    // }
}

// ! --------------  Tests --------------
