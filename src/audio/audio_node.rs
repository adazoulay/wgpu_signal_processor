use super::audio_clip::{AudioClip, AudioClipTrait};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct AudioNode<F> {
    pub name: Option<String>,
    clip: Arc<Mutex<AudioClip<F>>>,
    delta_clip: Arc<Mutex<AudioClip<F>>>,
    prev_clip: Arc<Mutex<AudioClip<F>>>,
    delta_range: Option<(usize, usize)>,
    clip_start: usize,
    clip_len: usize,
    // effect_chain: Option<AudioEffectChain<F>>,
}

impl<F> AudioNode<F>
where
    F: dasp::Frame<Sample = f32> + Copy,
{
    pub fn new(clip: AudioClip<F>, name: Option<&str>) -> Self {
        let name = name.map(|s| s.to_string());
        let prev_clip = clip.clone();
        let delta_clip = AudioClip::with_capacity(clip.get_length());
        let clip_len = clip.get_length();
        AudioNode {
            name,
            clip: Arc::new(Mutex::new(clip)),
            prev_clip: Arc::new(Mutex::new(prev_clip)),
            delta_clip: Arc::new(Mutex::new(delta_clip)),
            delta_range: None,
            clip_start: 0,
            clip_len,
            // effect_chain: None,
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

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    pub fn get_clip_start(&self) -> usize {
        self.clip_start
    }

    pub fn get_clip_len(&self) -> usize {
        self.clip_len
    }

    pub fn set_clip_len(&mut self, length: usize) {
        self.clip_len = length;
    }

    pub fn set_clip_start(&mut self, start_time: usize) {
        self.clip_start = start_time;
    }

    pub fn get_delta_range(&self) -> Option<(usize, usize)> {
        self.delta_range
    }

    pub fn get_absolute_delta_range(&self) -> Option<(usize, usize)> {
        if let Some((start, end)) = self.delta_range {
            return Some((start + self.clip_start, end + self.clip_start));
        }
        None
    }

    pub fn set_delta_range(&mut self, new_range: Option<(usize, usize)>) {
        self.delta_range = new_range;
    }

    pub fn resize_clips(&mut self, new_size: usize, value: F) {
        self.get_clip().resize_frames(new_size, value);
        self.get_delta_clip().resize_frames(new_size, value);
        self.get_prev_clip().resize_frames(new_size, value);
    }

    pub fn add_padding_left(&mut self, padding_amount: usize) {
        self.get_clip().add_padding_left(padding_amount);
        self.get_delta_clip().add_padding_left(padding_amount);
        self.get_prev_clip().add_padding_left(padding_amount);
    }

    pub fn normalize_clip_bounds(&mut self, parent_node: &AudioNode<F>) -> (usize, usize) {
        let parent_start = parent_node.get_clip_start();
        let parent_end = parent_start + parent_node.get_clip_len();

        let mut child_start = self.clip_start;
        let child_len = self.clip_len;
        let mut child_end = child_start + self.clip_len;

        if child_start > parent_start {
            let padding_amount = child_start - parent_start;
            self.add_padding_left(padding_amount);
            self.set_clip_len(child_len + padding_amount);
            self.set_clip_start(parent_start);
            child_start = parent_start;
        }

        if child_end < parent_end {
            let additional_len = parent_end - child_end;
            self.resize_clips(child_len + additional_len, F::EQUILIBRIUM);
            self.set_clip_len(child_len + additional_len);
            child_end = parent_end;
        }

        let overlap_start = std::cmp::max(parent_start, child_start);
        let overlap_end = std::cmp::min(parent_end, child_end);

        self.set_delta_range(Some((
            overlap_start - self.clip_start,
            overlap_end - self.clip_start,
        )));

        (overlap_start, overlap_end)
    }

    pub fn apply_delta(&mut self, parent_node: &AudioNode<F>) {
        if let Some((overlap_start, overlap_end)) = self.get_absolute_delta_range() {
            let child_clip_start = self.get_clip_start();
            let parent_delta_start = parent_node.get_clip_start();

            let parent_delta = parent_node.get_delta_clip();
            let mut child_clip = self.get_clip();

            let child_samples: &mut [F] = child_clip.get_frames_mut();
            let delta_samples: &[F] = parent_delta.get_frames_ref();

            for i in overlap_start..overlap_end {
                let delta_index = i - parent_delta_start;
                let child_index = i - child_clip_start;

                child_samples[child_index] =
                    (child_samples[child_index].add_amp(delta_samples[delta_index])).into();
            }
        }
    }

    pub fn compute_delta(&self) {
        if let Some((start, end)) = self.get_delta_range() {
            let prev_clip = self.get_prev_clip();
            let original_frames = prev_clip.get_frames_ref();

            let current_clip = self.get_clip();
            let current_frames = current_clip.get_frames_ref();

            let mut delta_clip = self.get_delta_clip();
            let delta_frames = delta_clip.get_frames_mut();

            for i in start..end {
                delta_frames[i] =
                    current_frames[i].add_amp(original_frames[i].scale_amp(-1.0 as f32));
            }
        }
    }

    pub fn commit_changes(&mut self) {
        self.set_delta_range(None);
        self.get_delta_clip().reset_clip();

        let mut prev_clip = self.get_prev_clip();
        let current_clip = self.get_clip();

        *prev_clip = current_clip.clone();
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
        let mut audio_node = create_mono_audio_node_with_samples(vec![1.0, 2.0, 3.0]);
        audio_node.resize_clips(5, [0.0]);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(clip_frames, vec![[1.0], [2.0], [3.0], [0.0], [0.0]]);
    }

    #[test]
    fn test_resize_clips_stereo() {
        let mut audio_node =
            create_stereo_audio_node_with_samples(vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        audio_node.resize_clips(5, [0.0, 0.0]);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(
            clip_frames,
            vec![[1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [0.0, 0.0], [0.0, 0.0]]
        );
    }

    #[test]
    fn test_add_padding_left_mono() {
        let mut audio_node = create_mono_audio_node_with_samples(vec![1.0, 2.0, 3.0]);
        audio_node.add_padding_left(2);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(clip_frames, vec![[0.0], [0.0], [1.0], [2.0], [3.0]]);
    }

    #[test]
    fn test_add_padding_left_stereo() {
        let mut audio_node =
            create_stereo_audio_node_with_samples(vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
        audio_node.add_padding_left(2);
        let clip_frames: Vec<_> = audio_node.get_clip().get_frames_ref().to_vec();
        assert_eq!(
            clip_frames,
            vec![[0.0, 0.0], [0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]
        );
    }

    #[test]
    fn test_normalize_clip_bounds() {
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(3);
        let (start, end) = child.normalize_clip_bounds(&parent);
        assert_eq!((start, end), (3, 6));

        let (absolute_start, absolute_end) = child.get_absolute_delta_range().unwrap();
        assert_eq!((absolute_start, absolute_end), (3, 6));
    }
    #[test]
    fn test_normalize_child_before_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(5);
        child.set_clip_start(0);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (5, 10));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (5, 10));
    }

    #[test]
    fn test_normalize_child_after_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(0);
        child.set_clip_start(6);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (0, 3));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 3));
    }

    #[test]
    fn test_normalize_child_overlapping_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);

        parent.set_clip_start(3);
        child.set_clip_start(1);

        let (start, end) = child.normalize_clip_bounds(&parent);
        assert_eq!((start, end), (3, 6));

        let (absolute_start, absolute_end) = child.get_absolute_delta_range().unwrap();
        assert_eq!((absolute_start, absolute_end), (3, 6));
    }

    #[test]
    fn test_normalize_child_same_start_as_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(0);
        child.set_clip_start(0);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (0, 3));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 3));
    }

    #[test]
    fn test_normalize_child_longer_than_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);

        parent.set_clip_start(0);
        child.set_clip_start(0);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (0, 3));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 3));
    }

    #[test]
    fn test_normalize_child_shorter_than_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(0);
        child.set_clip_start(0);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (0, 5));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 5));
    }

    #[test]
    fn test_normalize_child_starts_in_middle_of_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(0);
        child.set_clip_start(2);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (0, 5));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 5));
    }

    #[test]
    fn test_normalize_child_starts_after_parent_non_zero_start() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]); // 5
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]); // 3

        parent.set_clip_start(2);
        child.set_clip_start(4);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (2, 7));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 5));
    }

    #[test]
    fn test_normalize_child_starts_before_parent_non_zero_start() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(4);
        child.set_clip_start(2);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (4, 9));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (2, 7));
    }

    #[test]
    fn test_normalize_child_starts_same_time_as_parent_non_zero_start() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(2);
        child.set_clip_start(2);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (2, 7));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 5));
    }

    #[test] // ! Normalize panics still
    fn test_normalize_child_completely_before_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(6);
        child.set_clip_start(0);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (6, 9));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (6, 9));
    }
    #[test]
    fn test_normalize_child_completely_after_parent() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(0);
        child.set_clip_start(6);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (0, 3));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 3));
    }

    #[test]
    fn test_normalize_child_same_length_different_non_zero_start() {
        let mut parent = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);
        let mut child = create_mono_audio_node_with_samples(vec![1.0, 1.0, 1.0]);

        parent.set_clip_start(4);
        child.set_clip_start(6);

        let (absolute_start, absolute_end) = child.normalize_clip_bounds(&parent);
        assert_eq!((absolute_start, absolute_end), (4, 7));

        let (start, end) = child.get_delta_range().unwrap();
        assert_eq!((start, end), (0, 3));
    }
}
