use super::audio_clip::AudioClip;
use super::audio_node::AudioNode;
use super::audio_processor::AudioProcessor;
use crate::audio::audio_clip::AudioClipTrait;
use dasp::Frame;
use dasp::Sample;
use std::fmt;
use std::sync::MutexGuard;

// Define a trait for audio operations
pub trait AudioOperation<F>: Send {
    fn apply(&self, parent_node: &AudioNode<F>, child_node: &AudioNode<F>);
}

// Separate Linear and Non-Linear operations
pub trait LinearOperation<F>: AudioOperation<F> {}
pub trait NonLinearOperation<F>: AudioOperation<F> {}

pub struct AudioGraphEdge<F> {
    pub operation: Box<dyn AudioOperation<F>>,
    description: &'static str,
}

impl<F> fmt::Display for AudioGraphEdge<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge({})", self.description)
    }
}

impl<F> AudioGraphEdge<F> {
    pub fn new<O: 'static + AudioOperation<F>>(operation: O, description: &'static str) -> Self {
        AudioGraphEdge {
            operation: Box::new(operation),
            description,
        }
    }

    pub fn apply(&self, parent_node: &AudioNode<F>, child_node: &AudioNode<F>) {
        self.operation.apply(parent_node, child_node)
    }
}

impl<F> fmt::Debug for AudioGraphEdge<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge({})", self.description)
    }
}

#[derive(Clone)]
pub struct AddOperation;
impl<F: Frame<Sample = f32> + Default + Copy> LinearOperation<F> for AddOperation {}
impl<F: Frame<Sample = f32> + Default + Copy> AudioOperation<F> for AddOperation {
    fn apply(&self, parent_node: &AudioNode<F>, child_node: &AudioNode<F>) {
        let (overlap_start, overlap_end) =
            AudioProcessor::<F>::normalize_clip_bounds(parent_node, child_node);

        let parent_clip: MutexGuard<'_, AudioClip<F>> = parent_node.get_clip();
        let mut child_clip: MutexGuard<'_, AudioClip<F>> = child_node.get_clip();

        let child_start = child_clip.get_start_time_frame();
        let parent_start = parent_clip.get_start_time_frame();

        let parent_samples: &[F] = parent_clip.get_frames_ref();
        let child_samples: &mut [F] = child_clip.get_frames_mut();

        for i in overlap_start..overlap_end {
            let parent_index = i - parent_start;
            let child_index = i - child_start;

            child_samples[child_index] =
                (child_samples[child_index].add_amp(parent_samples[parent_index])).into();
        }
    }
}
