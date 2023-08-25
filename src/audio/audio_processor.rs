use super::audio_clip::{AudioClip, AudioClipEnum};
use super::audio_edge::{AddOperation, AudioGraphEdge};
use super::audio_graph::AudioGraph;
use super::audio_node::AudioNode;
use crate::audio::audio_clip::AudioClipTrait;
use dasp::frame::{Mono, Stereo};

use petgraph::stable_graph::{EdgeIndex, NodeIndex};
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
        self.root_frame_idx += 1;

        let audio_graph = self.lock_audio_graph();
        let root_clip = audio_graph
            .get_node_ref(self.root_node_index)
            .unwrap()
            .get_clip();
        root_clip.get_frame(self.root_frame_idx - 1)
    }

    pub fn set_root_frame_idx(&mut self, idx: usize) {
        self.root_frame_idx = idx;
    }

    //Todo Return result if connection is valid or invalid
    // Example, connection already exists or creates a cycle
    pub fn connect(
        &mut self,
        parent: NodeIndex,
        child: Option<NodeIndex>,
        edge: AudioGraphEdge<F>,
    ) -> EdgeIndex {
        let mut graph = self.lock_audio_graph();
        let edge_id = graph.connect(parent, child, edge).unwrap();

        let child_node_index = child.unwrap_or(graph.root);
        self.apply_effect(&mut graph, parent, child_node_index, edge_id);
        self.propagate_change(&mut graph, parent);
        edge_id
    }

    pub fn apply_effect(
        &self,
        audio_graph: &mut AudioGraph<F>,
        parent_idx: NodeIndex,
        child_idx: NodeIndex,
        edge_idx: EdgeIndex,
    ) {
        let parent_node = audio_graph
            .get_node_ref(parent_idx)
            .expect("Parent node not found");

        let child_node = audio_graph
            .get_node_ref(child_idx)
            .expect("Target node not found");

        let effect = audio_graph.get_edge_ref(edge_idx).unwrap();

        effect.operation.apply(parent_node, child_node);
    }

    pub fn propagate_change(&self, audio_graph: &mut AudioGraph<F>, node_idx: NodeIndex) {
        let to_compute = audio_graph.collect_dependents(node_idx);
        let mut last_child_node = None;

        for (parent, child, _edge) in to_compute {
            let parent_node = audio_graph
                .get_node_ref(parent)
                .expect("Parent node not found");
            let child_node = audio_graph
                .get_node_ref(child)
                .expect("Child node not found");

            AudioProcessor::<F>::apply_delta(parent_node, child_node);
            child_node.compute_delta();
            parent_node.commit_changes();

            last_child_node = Some(child_node);
        }

        if let Some(node) = last_child_node {
            node.commit_changes();
        }
    }

    pub fn normalize_clip_bounds(
        parent_node: &AudioNode<F>,
        child_node: &AudioNode<F>,
    ) -> (usize, usize) {
        let parent_clip = parent_node.get_clip();
        let child_clip = child_node.get_delta_clip();

        let parent_start = parent_clip.get_start_time_frame() as usize;
        let mut child_start = child_clip.get_start_time_frame() as usize;

        let parent_end = parent_start + parent_clip.get_length() as usize;
        let mut child_end = child_start + child_clip.get_length() as usize;

        if child_start > parent_start {
            child_node.add_padding_left(parent_start);
            child_node.set_start_time_frame(parent_start);
            child_start = parent_start;
        }

        let new_child_end = std::cmp::max(child_end, parent_end);
        if new_child_end > child_clip.get_length() {
            child_node.resize_clips(new_child_end, F::EQUILIBRIUM);
            child_end = new_child_end;
        }

        let overlap_start = std::cmp::max(parent_start, child_start);
        let overlap_end = std::cmp::min(parent_end, child_end);

        (overlap_start, overlap_end)
    }

    fn apply_delta(source_node: &AudioNode<F>, target_node: &AudioNode<F>) {
        let (overlap_start, overlap_end) =
            AudioProcessor::normalize_clip_bounds(source_node, target_node);

        let source_delta = source_node.get_delta_clip();
        let mut target_clip = target_node.get_clip();

        let target_clip_start = target_clip.get_start_time_frame();
        let source_delta_start = source_delta.get_start_time_frame();

        let target_samples: &mut [F] = target_clip.get_frames_mut();
        let delta_samples: &[F] = source_delta.get_frames_ref();

        for i in overlap_start..overlap_end {
            let delta_index = i - source_delta_start;
            let target_index = i - target_clip_start;

            target_samples[target_index] =
                (target_samples[target_index].add_amp(delta_samples[delta_index])).into();
        }
    }

    fn get_node_frames_copy(&self, node_index: NodeIndex) -> Vec<F> {
        let graph = self.lock_audio_graph();
        let node = graph.get_node_ref(node_index).unwrap();
        let x = node.get_clip().get_frames_ref().clone().to_vec();
        x
    }

    fn print_graph(&self) {
        self.lock_audio_graph().print_graph();
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
    use dasp::frame::Mono;

    fn create_simple_clip() -> AudioClip<Mono<f32>> {
        let samples: Vec<f32> = (1..=3).map(|x| x as f32).collect();
        AudioClip::<Mono<f32>>::new(samples, 44100)
    }

    #[test]
    fn test_two_nodes_to_root() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let node1 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node1"));
        let node2 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node2"));

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node1, None, add_edge);
        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node2, None, add_edge);

        let expected_samples = [[2.0], [4.0], [6.0]];

        for expected in &expected_samples {
            let sample = processor.get_root_sample().expect("Expected a sample");
            assert_eq!(sample, *expected);
        }
    }

    #[test]
    fn test_audio_processor_complex_graph() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let node1 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node1"));
        let node2 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node2"));
        let node3 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node3"));
        let node4 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node4"));

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node1, Some(node3), add_edge);
        let frames = processor.get_node_frames_copy(node3);
        let expected_frames = vec![[2.0], [4.0], [6.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node2, Some(node3), add_edge);
        let frames = processor.get_node_frames_copy(node3);
        let expected_frames = vec![[3.0], [6.0], [9.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node3, Some(node4), add_edge);
        let frames = processor.get_node_frames_copy(node4);
        let expected_frames = vec![[4.0], [8.0], [12.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node4, None, add_edge);

        let expected_samples = [[4.0], [8.0], [12.0]];

        for expected in &expected_samples {
            let sample = processor.get_root_sample().expect("Expected a sample");
            assert_eq!(sample, *expected);
        }
    }

    #[test]
    fn test_audio_processor_complex_graph_2() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        // Create nodes
        let node1 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node1"));
        let node2 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node2"));
        let node3 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node3"));
        let node4 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node4"));
        let node5 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node5"));
        let node6 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node6"));
        let node7 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node7"));
        let node8 = processor.add_node(AudioClipEnum::Mono(create_simple_clip()), Some("node8"));

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node1, Some(node3), add_edge);
        let frames = processor.get_node_frames_copy(node3);
        let expected_frames = vec![[2.0], [4.0], [6.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node2, Some(node3), add_edge);
        let frames = processor.get_node_frames_copy(node3);
        let expected_frames = vec![[3.0], [6.0], [9.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node3, Some(node4), add_edge);
        let frames = processor.get_node_frames_copy(node4);
        let expected_frames = vec![[4.0], [8.0], [12.0]]; // node3 + node4
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node5, Some(node4), add_edge);
        let frames = processor.get_node_frames_copy(node4);
        let expected_frames = vec![[5.0], [10.0], [15.0]]; // previous node4 + node5
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node6, Some(node5), add_edge);
        let frames = processor.get_node_frames_copy(node5);
        let expected_frames = vec![[2.0], [4.0], [6.0]]; // original node5 + node6
        assert_eq!(frames, expected_frames);
        let frames = processor.get_node_frames_copy(node4);
        let expected_frames = vec![[6.0], [12.0], [18.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node7, Some(node5), add_edge);
        let frames = processor.get_node_frames_copy(node5);
        let expected_frames = vec![[3.0], [6.0], [9.0]]; // previous node5 + node7
        assert_eq!(frames, expected_frames);
        let frames = processor.get_node_frames_copy(node4);
        let expected_frames = vec![[7.0], [14.0], [21.0]];
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node4, None, add_edge);

        processor.print_graph();

        let expected_samples = [[7.0], [14.0], [21.0]];

        for expected in &expected_samples {
            let sample = processor.get_root_sample().expect("Expected a sample");
            assert_eq!(sample, *expected);
        }

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node8, Some(node5), add_edge);
        let frames = processor.get_node_frames_copy(node5);
        let expected_frames = vec![[4.0], [8.0], [12.0]]; // previous node5 + node8
        assert_eq!(frames, expected_frames);
        let frames = processor.get_node_frames_copy(node4);
        let expected_frames = vec![[8.0], [16.0], [24.0]];
        assert_eq!(frames, expected_frames);

        let expected_samples = [[8.0], [16.0], [24.0]];
        processor.set_root_frame_idx(0);
        for expected in &expected_samples {
            let sample = processor.get_root_sample().expect("Expected a sample");
            assert_eq!(sample, *expected);
        }
    }

    fn create_clip_with_size_start(size: usize, start: usize) -> AudioClip<Mono<f32>> {
        let mut frames = Vec::with_capacity(size);
        for _ in 0..size {
            frames.push(1.0);
        }
        let mut clip = AudioClip::<Mono<f32>>::new(frames, 44100);
        clip.set_start_time_frame(start);
        clip
    }
    #[test]
    fn test_normalize_clip_bounds() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let clip1 = create_clip_with_size_start(10, 0);
        let clip2 = create_clip_with_size_start(10, 5);

        let clip3 = create_clip_with_size_start(10, 5);
        let clip4 = create_clip_with_size_start(10, 0);

        let node_idx1 = processor.add_node(AudioClipEnum::Mono(clip1), Some("node1"));
        let node_idx2 = processor.add_node(AudioClipEnum::Mono(clip2), Some("node2"));
        let node_idx3 = processor.add_node(AudioClipEnum::Mono(clip3), Some("node3"));
        let node_idx4 = processor.add_node(AudioClipEnum::Mono(clip4), Some("node4"));

        let graph = processor.lock_audio_graph();

        let node1 = graph.get_node_ref(node_idx1).unwrap();
        let node2 = graph.get_node_ref(node_idx2).unwrap();

        let (overlap_start, overlap_end) =
            AudioProcessor::<Mono<f32>>::normalize_clip_bounds(node1, node2);

        assert_eq!((overlap_start, overlap_end), (0, 10));
    }
}
