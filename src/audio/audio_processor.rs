use super::audio_clip::{AudioClip, AudioClipEnum};
use super::audio_graph::AudioGraph;
use super::audio_node::AudioNode;
use crate::audio::audio_clip::AudioClipTrait;
use crate::audio::audio_graph::AudioGraphEdge;
use dasp::frame::{Mono, Stereo};
use dasp::Frame;
use std::collections::HashMap;

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct AudioProcessor<F> {
    audio_graph: Arc<Mutex<AudioGraph<F>>>,
    root_frame_idx: usize,
    root_node_index: NodeIndex,
    sample_rate: u32,
}

impl<F> AudioProcessor<F>
where
    F: dasp::Frame<Sample = f32> + Default + Copy,
{
    pub fn new() -> Self {
        let audio_graph = AudioGraph::new();
        let root_node_index = audio_graph.root;
        Self {
            audio_graph: Arc::new(Mutex::new(audio_graph)),
            root_frame_idx: 0,
            root_node_index,
            sample_rate: 44100,
        }
    }

    pub fn lock_audio_graph(&self) -> std::sync::MutexGuard<AudioGraph<F>> {
        self.audio_graph.lock().unwrap()
    }

    pub fn get_root_sample(&mut self) -> Option<F> {
        let audio_graph = self.lock_audio_graph();
        let root_clip = audio_graph.get_clip(self.root_node_index).unwrap();
        root_clip.get_frame(self.root_frame_idx)
    }

    pub fn connect(&mut self, from: NodeIndex, to: Option<NodeIndex>, op: AudioGraphEdge) {
        self.lock_audio_graph().connect(from, to, op);
        self.propagate_change(from);
    }

    pub fn propagate_change(&mut self, node_idx: NodeIndex) {
        let to_compute = {
            self.audio_graph
                .lock()
                .unwrap()
                .collect_dependents(node_idx)
        };

        for (parent, child, edge) in to_compute {
            self.compute(parent, child, edge);
        }
    }

    pub fn compute(&self, parent: NodeIndex, child: NodeIndex, edge: AudioGraphEdge) {
        let mut audio_graph = self.audio_graph.lock().unwrap();
        self.apply_effect(&mut audio_graph, parent, child, edge);
    }

    pub fn apply_effect(
        &self,
        graph: &mut AudioGraph<F>,
        parent_idx: NodeIndex,
        child_idx: NodeIndex,
        effect: AudioGraphEdge,
    ) {
        let parent_node = graph.get_clip(parent_idx).expect("Parent node not found");

        let child_node = graph.get_clip(child_idx).expect("Target node not found");

        match effect {
            AudioGraphEdge::Add => AudioProcessor::<F>::add_samples(parent_node, child_node),
            _ => {}
        }
    }

    fn add_samples(parent: MutexGuard<'_, AudioClip<F>>, mut child: MutexGuard<'_, AudioClip<F>>) {
        let parent_start = parent.get_start_time_frame() as usize;
        let child_start = child.get_start_time_frame() as usize;

        let parent_end = parent_start + parent.get_length() as usize;
        let child_end = child_start + child.get_length() as usize;

        // Calculate the overlapping range
        let overlap_start = std::cmp::max(parent_start, child_start);
        let overlap_end = std::cmp::min(parent_end, child_end);

        // Determine the new end for the child (accounting for possible extension due to the parent)
        let new_child_end = std::cmp::max(child_end, parent_end);

        if new_child_end > child.get_length() {
            child.resize_frames(new_child_end, F::EQUILIBRIUM);
        }

        let parent_samples = parent.get_frames_ref();
        let child_samples = child.get_frames_mut();

        if overlap_start < overlap_end {
            // Overlap
            for i in overlap_start..overlap_end {
                let parent_index = i - parent_start;
                let child_index = i - child_start;

                child_samples[child_index] =
                    (child_samples[child_index].add_amp(parent_samples[parent_index])).into();
            }
        } else {
            // No overlap
            for i in parent_start..parent_end {
                let parent_index = i - parent_start;
                let child_index = i - child_start;

                child_samples[child_index] = parent_samples[parent_index].clone();
            }
        }
    }
}

impl AudioProcessor<Mono<f32>> {
    pub fn add_node(&mut self, clip: AudioClipEnum, name: Option<&str>) -> NodeIndex {
        let mut clip = match clip {
            AudioClipEnum::Mono(clip) => clip,
            AudioClipEnum::Stereo(clip) => clip.to_mono(),
        };

        if self.sample_rate != clip.get_sample_rate() {
            clip = clip.resample(self.sample_rate);
        }
        let audio_node = AudioNode::new(clip, name);
        self.lock_audio_graph().add_data_node(audio_node)
    }
}

impl AudioProcessor<Stereo<f32>> {
    pub fn add_node(&mut self, clip: AudioClipEnum, name: Option<&str>) -> NodeIndex {
        let mut clip = match clip {
            AudioClipEnum::Mono(clip) => clip.to_stereo(),
            AudioClipEnum::Stereo(clip) => clip,
        };

        if self.sample_rate != clip.get_sample_rate() {
            clip = clip.resample(self.sample_rate);
        }
        let audio_node = AudioNode::new(clip, name);
        self.lock_audio_graph().add_data_node(audio_node)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dasp::frame::Frame;
    use dasp::frame::{Mono, Stereo};

    #[test]
    fn test_connect_and_propagate_mono() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let node1 = processor.add_node(AudioClipEnum::Mono(create_mock_clip()), Some("node1"));
        let node2 = processor.add_node(AudioClipEnum::Mono(create_mock_clip()), Some("node2"));

        // Validate initial node values
        validate_initial_clip_values(&processor, node1);
        validate_initial_clip_values(&processor, node2);

        processor.connect(node1, Some(node2), AudioGraphEdge::Add);
        processor.connect(node2, None, AudioGraphEdge::Add);

        // Validate post-connect values
        validate_connected_clip_values(&mut processor);
    }

    #[test]
    fn test_connect_and_propagate_stereo() {
        let mut processor = AudioProcessor::<Stereo<f32>>::new();

        let node1 = processor.add_node(
            AudioClipEnum::Stereo(create_mock_clip_stereo()),
            Some("node1"),
        );
        let node2 = processor.add_node(
            AudioClipEnum::Stereo(create_mock_clip_stereo()),
            Some("node2"),
        );

        // Validate initial node values
        validate_initial_clip_values_stereo(&processor, node1);
        validate_initial_clip_values_stereo(&processor, node2);

        processor.connect(node1, Some(node2), AudioGraphEdge::Add);
        processor.connect(node2, None, AudioGraphEdge::Add);

        // Validate post-connect values
        validate_connected_clip_values_stereo(&mut processor);
    }

    #[test]
    fn test_complex_audio_graph_mono() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let node1 = processor.add_node(AudioClipEnum::Mono(create_mock_clip()), Some("node1"));
        let node2 = processor.add_node(AudioClipEnum::Mono(create_mock_clip()), Some("node2"));
        let node3 = processor.add_node(AudioClipEnum::Mono(create_mock_clip()), Some("node3"));
        let node4 = processor.add_node(AudioClipEnum::Mono(create_mock_clip()), Some("node4"));

        processor.connect(node1, Some(node3), AudioGraphEdge::Add);
        processor.connect(node2, Some(node3), AudioGraphEdge::Add);
        processor.connect(node3, Some(node4), AudioGraphEdge::Add);
        processor.connect(node4, None, AudioGraphEdge::Add);

        processor.lock_audio_graph().print_graph();
        let expected_samples: Vec<[f32; 1]> = (1..=1000).map(|x| [x as f32 * 4.0]).collect();

        for i in 0..999 {
            let sample = processor.get_root_sample().unwrap();
            assert_eq!(sample, expected_samples[i]);
        }
    }

    fn validate_initial_clip_values(processor: &AudioProcessor<Mono<f32>>, node: NodeIndex) {
        let audio_graph = processor.lock_audio_graph();
        let clip = audio_graph.get_clip(node).expect("Node not found");
        for i in 0..1000 {
            let sample = clip.get_frame(i).unwrap();
            assert_eq!(sample, [i as f32 + 1.0]);
        }
    }

    fn validate_initial_clip_values_stereo(
        processor: &AudioProcessor<Stereo<f32>>,
        node: NodeIndex,
    ) {
        let audio_graph = processor.lock_audio_graph();
        let clip = audio_graph.get_clip(node).expect("Node not found");
        for i in 0..1000 {
            let sample = clip.get_frame(i).unwrap();
            assert_eq!(sample, [i as f32 + 1.0, i as f32 + 1.0]);
        }
    }

    fn validate_connected_clip_values(processor: &mut AudioProcessor<Mono<f32>>) {
        for i in 0..1000 {
            let sample = processor.get_root_sample().unwrap();
            assert_ne!(sample, Mono::<f32>::EQUILIBRIUM);
        }
    }

    fn validate_connected_clip_values_stereo(processor: &mut AudioProcessor<Stereo<f32>>) {
        for i in 0..1000 {
            let sample = processor.get_root_sample().unwrap();
            assert_ne!(sample, Stereo::<f32>::EQUILIBRIUM);
        }
    }

    fn create_mock_clip() -> AudioClip<Mono<f32>> {
        let samples: Vec<f32> = (1..=1000).map(|x| x as f32).collect();
        AudioClip::<Mono<f32>>::new(samples, 44100)
    }

    fn create_mock_clip_stereo() -> AudioClip<Stereo<f32>> {
        let samples: Vec<f32> = (1..=1000).flat_map(|x| vec![x as f32, x as f32]).collect();
        AudioClip::<Stereo<f32>>::new(samples, 44100)
    }
}
