use super::audio_clip::{AudioClip, AudioClipTrait};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct AudioNode<F> {
    pub name: String,
    clip: Arc<Mutex<AudioClip<F>>>,
    delta_clip: Arc<Mutex<AudioClip<F>>>,
    prev_clip: Arc<Mutex<AudioClip<F>>>,
    modified_range: Mutex<Option<(usize, usize)>>,
    clip_start: usize,
    clip_len: usize,
    // effect_chain: Option<AudioEffectChain<F>>,
}

impl<F> AudioNode<F>
where
    F: dasp::Frame<Sample = f32> + Copy,
{
    //Todo Maybe pass in AudioClip and remove option name since Graph takes care of naming
    pub fn new(clip: AudioClip<F>, name: Option<&str>) -> Self {
        let name = name.unwrap_or("default").to_string();
        let prev_clip = clip.clone();
        let delta_clip = AudioClip::with_capacity(clip.get_length());
        let clip_start = clip.get_start_time_frame();
        let clip_len = clip.get_length();
        AudioNode {
            name,
            clip: Arc::new(Mutex::new(clip)),
            prev_clip: Arc::new(Mutex::new(prev_clip)),
            delta_clip: Arc::new(Mutex::new(delta_clip)),
            modified_range: Mutex::new(None),
            clip_start,
            clip_len, // effect_chain: None,
        }
    }

    pub fn get_clip(&self) -> MutexGuard<'_, AudioClip<F>> {
        self.clip.lock().unwrap()
    }

    pub fn get_delta_clip(&self) -> MutexGuard<'_, AudioClip<F>> {
        self.delta_clip.lock().unwrap()
    }

    pub fn get_prev_clip(&self) -> MutexGuard<'_, AudioClip<F>> {
        self.prev_clip.lock().unwrap()
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    pub fn get_modified_range(&self) -> Option<(usize, usize)> {
        *self.modified_range.lock().unwrap()
    }

    pub fn set_modified_range(&self, new_range: Option<(usize, usize)>) {
        *self.modified_range.lock().unwrap() = new_range;
    }

    pub fn resize_clips(&self, new_size: usize, value: F) {
        self.get_clip().resize_frames(new_size, value);
        self.get_delta_clip().resize_frames(new_size, value);
        self.get_prev_clip().resize_frames(new_size, value); // Maybe don't mutate origianl?
    }

    pub fn add_padding_left(&self, new_start_time: usize) {
        let old_start_time = self.get_clip().get_start_time_frame() as usize;
        let padding_frames = new_start_time - old_start_time;
        self.get_clip().add_padding_left(padding_frames);
        self.get_delta_clip().add_padding_left(padding_frames);
        self.get_prev_clip().add_padding_left(padding_frames);
    }

    pub fn set_start_time_frame(&self, start_time: usize) {
        self.get_clip().set_start_time_frame(start_time);
        self.get_delta_clip().set_start_time_frame(start_time);
        self.get_prev_clip().set_start_time_frame(start_time);
    }

    // pub fn compute_modified_range(&self, parent_node: &AudioNode<F>) {
    //     let parent_clip = parent_node.get_clip();
    //     let child_clip = self.get_delta_clip();

    //     let parent_start = parent_clip.get_start_time_frame() as usize;
    //     let mut child_start = child_clip.get_start_time_frame() as usize;

    //     let parent_end = parent_start + parent_clip.get_length() as usize;
    //     let mut child_end = child_start + child_clip.get_length() as usize;

    //     if child_start > parent_start {
    //         self.add_padding_left(parent_start);
    //         child_node.set_start_time_frame(parent_start);
    //         child_start = parent_start;
    //     }

    //     let new_child_end = std::cmp::max(child_end, parent_end);
    //     if new_child_end > child_clip.get_length() {
    //         child_node.resize_clips(new_child_end, F::EQUILIBRIUM);
    //         child_end = new_child_end;
    //     }

    //     let overlap_start = std::cmp::max(parent_start, child_start);
    //     let overlap_end = std::cmp::min(parent_end, child_end);

    //     (overlap_start, overlap_end)
    // }

    pub fn compute_delta(&self) {
        // if let Some((start, end)) = self.get_modified_range() {
        let prev_clip = self.get_prev_clip();
        let original_frames = prev_clip.get_frames_ref();

        let current_clip = self.get_clip();
        let current_frames = current_clip.get_frames_ref();

        let mut delta_clip = self.get_delta_clip();
        let delta_frames = delta_clip.get_frames_mut();

        for i in 0..current_clip.get_length() {
            delta_frames[i] = current_frames[i].add_amp(original_frames[i].scale_amp(-1.0 as f32));
        }
        // }
    }

    pub fn commit_changes(&self) {
        let mut prev_clip = self.get_prev_clip();
        let current_clip = self.get_clip();

        *prev_clip = current_clip.clone();

        self.reset_delta();
        self.set_modified_range(None);
    }

    pub fn reset_delta(&self) {
        self.get_delta_clip().reset_clip();
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

#[cfg(test)]
mod tests {
    use super::*;
    use dasp::frame::Mono;
    use dasp::frame::Stereo;

    // Helper functions
    fn create_mono_audio_node_with_samples(samples: Vec<f32>) -> AudioNode<Mono<f32>> {
        let clip = AudioClip::<Mono<f32>>::new(samples, 44100);
        AudioNode::new(clip, Some("mono_test_clip"))
    }

    fn create_stereo_audio_node_with_samples(samples: Vec<f32>) -> AudioNode<Stereo<f32>> {
        let clip = AudioClip::<Stereo<f32>>::new(samples, 44100);
        AudioNode::new(clip, Some("stereo_test_clip"))
    }

    // resize_clips tests
    #[test]
    fn test_resize_clips_mono() {
        let audio_node = create_mono_audio_node_with_samples(vec![1.0, 2.0, 3.0]);
        audio_node.resize_clips(5, [0.0]);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(clip_frames, vec![[1.0], [2.0], [3.0], [0.0], [0.0]]);
    }

    #[test]
    fn test_resize_clips_stereo() {
        let audio_node = create_stereo_audio_node_with_samples(vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        audio_node.resize_clips(5, [0.0, 0.0]);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(
            clip_frames,
            vec![[1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [0.0, 0.0], [0.0, 0.0]]
        );
    }

    // add_padding_left tests
    #[test]
    fn test_add_padding_left_mono() {
        let audio_node = create_mono_audio_node_with_samples(vec![1.0, 2.0, 3.0]);
        audio_node.add_padding_left(2);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(clip_frames, vec![[0.0], [0.0], [1.0], [2.0], [3.0]]);
    }

    #[test]
    fn test_add_padding_left_stereo() {
        let audio_node = create_stereo_audio_node_with_samples(vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        audio_node.add_padding_left(2);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(
            clip_frames,
            vec![[0.0, 0.0], [0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]
        );
    }

    // ... Continue with the other tests ...
}
