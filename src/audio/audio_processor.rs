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
    pub root_node_index: NodeIndex,
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

    pub fn lock_audio_graph(&self) -> MutexGuard<AudioGraph<F>> {
        self.audio_graph.lock().unwrap()
    }

    pub fn get_node_or_root_sample(&mut self, node: Option<NodeIndex>) -> Option<F> {
        self.root_frame_idx += 1;
        let node_idx = node.unwrap_or(self.root_node_index);
        let audio_graph = self.lock_audio_graph();

        let root_node = audio_graph.get_node(node_idx).unwrap().lock().unwrap();
        let root_clip = root_node.get_clip();

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
            .get_node(parent_idx)
            .expect("Parent node not found")
            .lock()
            .unwrap();

        let effect = audio_graph.get_edge_ref(edge_idx).unwrap();

        let mut child_node = audio_graph
            .get_node(child_idx)
            .expect("Target node not found")
            .lock()
            .unwrap();

        child_node.normalize_clip_bounds(&*parent_node);

        effect.operation.apply(&*parent_node, &*child_node);
    }

    pub fn propagate_change(&self, audio_graph: &mut AudioGraph<F>, node_idx: NodeIndex) {
        let to_compute = audio_graph.collect_dependents(node_idx);
        let mut last_child_node = None;

        for (parent, child, _edge) in to_compute {
            let mut parent_node = audio_graph
                .get_node(parent)
                .expect("Parent node not found")
                .lock()
                .unwrap();

            let mut child_node = audio_graph
                .get_node(child)
                .expect("Child node not found")
                .lock()
                .unwrap();

            child_node.normalize_clip_bounds(&*parent_node);
            child_node.apply_delta(&*parent_node);
            child_node.compute_delta();
            parent_node.commit_changes();

            last_child_node = Some(child);
        }

        if let Some(child_idx) = last_child_node {
            audio_graph
                .get_node(child_idx)
                .expect("Child node not found")
                .lock()
                .unwrap()
                .commit_changes();
        }
    }

    pub fn add_node(&mut self, node: AudioNode<F>) -> NodeIndex {
        self.lock_audio_graph().add_data_node(node)
    }

    fn get_node_frames_copy(&self, node_index: NodeIndex) -> Vec<F> {
        let graph = self.lock_audio_graph();
        let node = graph.get_node(node_index).unwrap().lock().unwrap();
        let x = node.get_clip().get_frames_ref().clone().to_vec();
        x
    }

    fn print_graph(&self) {
        self.lock_audio_graph().print_graph();
    }
}

impl AudioProcessor<Mono<f32>> {
    pub fn add_node_from_clip(&mut self, clip: AudioClipEnum, name: Option<&str>) -> NodeIndex {
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
    pub fn add_node_from_clip(&mut self, clip: AudioClipEnum, name: Option<&str>) -> NodeIndex {
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

        let node1 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node1"));
        let node2 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node2"));

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node1, None, add_edge);
        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node2, None, add_edge);

        let expected_samples = [[2.0], [4.0], [6.0]];

        for expected in &expected_samples {
            let sample = processor
                .get_node_or_root_sample(None)
                .expect("Expected a sample");
            assert_eq!(sample, *expected);
        }
    }

    #[test]
    fn test_audio_processor_complex_graph() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let node1 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node1"));
        let node2 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node2"));
        let node3 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node3"));
        let node4 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node4"));

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
            let sample = processor
                .get_node_or_root_sample(None)
                .expect("Expected a sample");
            assert_eq!(sample, *expected);
        }
    }

    #[test]
    fn test_audio_processor_complex_graph_2() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        // Create nodes
        let node1 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node1"));
        let node2 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node2"));
        let node3 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node3"));
        let node4 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node4"));
        let node5 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node5"));
        let node6 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node6"));
        let node7 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node7"));
        let node8 =
            processor.add_node_from_clip(AudioClipEnum::Mono(create_simple_clip()), Some("node8"));

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
            let sample = processor
                .get_node_or_root_sample(None)
                .expect("Expected a sample");
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
            let sample = processor
                .get_node_or_root_sample(None)
                .expect("Expected a sample");
            assert_eq!(sample, *expected);
        }
    }

    fn create_unit_node(size: usize) -> AudioNode<Mono<f32>> {
        let mut frames = Vec::with_capacity(size);
        for _ in 0..size {
            frames.push(1.0);
        }
        let clip = AudioClip::<Mono<f32>>::new(frames, 44100);
        AudioNode::new(clip, None)
    }
    #[test]
    fn test_normalize_clip_bounds() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let node1 = create_unit_node(3);
        let mut node2 = create_unit_node(3);

        let mut node3 = create_unit_node(5);
        let mut node4 = create_unit_node(3);

        node2.set_clip_start(4);
        node3.set_clip_start(0);
        node4.set_clip_start(7);

        let node_id1 = processor.add_node(node1);
        let node_id2 = processor.add_node(node2);
        let node_id3 = processor.add_node(node3);
        let node_id4 = processor.add_node(node4);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node_id1, Some(node_id2), add_edge);
        let frames = processor.get_node_frames_copy(node_id2);
        let expected_frames = vec![[1.0], [1.0], [1.0], [0.0], [1.0], [1.0], [1.0]]; // previous node5 + node8
        assert_eq!(frames, expected_frames);

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        processor.connect(node_id3, Some(node_id4), add_edge);

        processor.print_graph();
        let frames = processor.get_node_frames_copy(node_id4);
        let expected_frames = vec![
            [1.0],
            [1.0],
            [1.0],
            [1.0],
            [1.0],
            [0.0],
            [0.0],
            [1.0],
            [1.0],
            [1.0],
        ]; // previous node5 + node8
        assert_eq!(frames, expected_frames);
    }

    #[test]
    fn test_complex_tree_normalize_clip_bounds() {
        let mut processor = AudioProcessor::<Mono<f32>>::new();

        let mut node1 = create_unit_node(3);
        let mut node2 = create_unit_node(5);
        let mut node3 = create_unit_node(3);
        let mut node4 = create_unit_node(3);
        let mut node5 = create_unit_node(5);

        node1.set_clip_start(0);
        node2.set_clip_start(4);
        node3.set_clip_start(0);
        node4.set_clip_start(7);
        node5.set_clip_start(2);

        let node_id1 = processor.add_node(node1); // 0-3
        let node_id2 = processor.add_node(node2); // 4-5
        let node_id3 = processor.add_node(node3); // 0-3
        let node_id4 = processor.add_node(node4); // 7-3
        let node_id5 = processor.add_node(node5); // 2-5

        // Connect Node5 directly to the Root Node
        let root_edge = AudioGraphEdge::new(AddOperation, "RootOp");
        processor.connect(node_id5, None, root_edge);

        let frames_node_root = processor.get_node_frames_copy(processor.root_node_index);
        let expected_frames_root = vec![[0.0], [0.0], [1.0], [1.0], [1.0], [1.0], [1.0], [0.0]]; // node5 + node1 from time frame 2
        assert_eq!(frames_node_root[0..8], expected_frames_root);

        // Connect Node1 and Node2 to Node5
        let add_edge1 = AudioGraphEdge::new(AddOperation, "AddOp1");
        processor.connect(node_id1, Some(node_id5), add_edge1);

        let frames_node1 = processor.get_node_frames_copy(node_id5);
        let expected_frames_node1 = vec![[1.0], [1.0], [2.0], [1.0], [1.0], [1.0], [1.0]]; // node5 + node1 from time frame 2
        assert_eq!(frames_node1, expected_frames_node1);

        let frames_node_root = processor.get_node_frames_copy(processor.root_node_index);
        let expected_frames_root = vec![[1.0], [1.0], [2.0], [1.0], [1.0], [1.0], [1.0], [0.0]]; // node5 + node1 from time frame 2
        assert_eq!(frames_node_root[0..8], expected_frames_root);

        let add_edge2 = AudioGraphEdge::new(AddOperation, "AddOp2");
        processor.connect(node_id2, Some(node_id5), add_edge2);

        let node_5 = processor.get_node_frames_copy(node_id5);
        let expected_node_5 = vec![
            [1.0],
            [1.0],
            [2.0],
            [1.0],
            [2.0],
            [2.0],
            [2.0],
            [1.0],
            [1.0],
        ]; // node5 + node1 from time frame 2
        assert_eq!(node_5, expected_node_5);

        let frames_node_root = processor.get_node_frames_copy(processor.root_node_index);
        let expected_frames_root = vec![
            [1.0],
            [1.0],
            [2.0],
            [1.0],
            [2.0],
            [2.0],
            [2.0],
            [1.0],
            [1.0],
            [0.0],
        ]; // node5 + node1 from time frame 2
        assert_eq!(frames_node_root[0..10], expected_frames_root);

        // Connect Node3 and Node4 to Node2
        let add_edge3 = AudioGraphEdge::new(AddOperation, "AddOp3");
        processor.connect(node_id3, Some(node_id2), add_edge3);

        let node_2 = processor.get_node_frames_copy(node_id2);
        let expected_node_2 = vec![
            [1.0],
            [1.0],
            [1.0],
            [0.0],
            [1.0],
            [1.0],
            [1.0],
            [1.0],
            [1.0],
        ];
        assert_eq!(node_2, expected_node_2);

        let node_5 = processor.get_node_frames_copy(node_id5);
        let expected_node_5 = vec![
            [2.0],
            [2.0],
            [3.0],
            [1.0],
            [2.0],
            [2.0],
            [2.0],
            [1.0],
            [1.0],
        ];
        assert_eq!(node_5, expected_node_5);

        let frames_node_root = processor.get_node_frames_copy(processor.root_node_index);
        let expected_frames_root = vec![
            [2.0],
            [2.0],
            [3.0],
            [1.0],
            [2.0],
            [2.0],
            [2.0],
            [1.0],
            [1.0],
            [0.0],
        ];
        assert_eq!(frames_node_root[0..10], expected_frames_root);

        let add_edge4 = AudioGraphEdge::new(AddOperation, "AddOp4");
        processor.connect(node_id4, Some(node_id2), add_edge4);

        let node_2 = processor.get_node_frames_copy(node_id2);
        let expected_node_2 = vec![
            [1.0],
            [1.0],
            [1.0],
            [0.0],
            [1.0],
            [1.0],
            [1.0],
            [2.0],
            [2.0],
            [1.0],
        ];
        assert_eq!(node_2, expected_node_2);

        let node_5 = processor.get_node_frames_copy(node_id5);
        let expected_node_5 = vec![
            [2.0],
            [2.0],
            [3.0],
            [1.0],
            [2.0],
            [2.0],
            [2.0],
            [2.0],
            [2.0],
            [1.0],
        ];
        assert_eq!(node_5, expected_node_5);

        let frames_node_root = processor.get_node_frames_copy(processor.root_node_index);
        let expected_frames_root = vec![
            [2.0],
            [2.0],
            [3.0],
            [1.0],
            [2.0],
            [2.0],
            [2.0],
            [2.0],
            [2.0],
            [1.0],
            [0.0],
        ];
        assert_eq!(frames_node_root[0..11], expected_frames_root);
        processor.print_graph();
    }
}
