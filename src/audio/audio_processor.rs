use super::audio_graph::AudioGraph;
use crate::audio::audio_clip::AudioClipTrait;
use crate::audio::audio_graph::AudioGraphEdge;
use crate::audio::audio_node::AudioNode;
use dasp::frame::{Mono, Stereo};
use dasp::Frame;

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::sync::{Arc, Mutex};

pub enum AudioFrame {
    Mono(Mono<f32>),
    Stereo(Stereo<f32>),
}

pub struct AudioProcessor {
    audio_graph: Arc<Mutex<AudioGraph>>,
    root_frame_idx: usize,
    root_node_index: NodeIndex,
    sample_rate: u32,
}

impl AudioProcessor {
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

    pub fn propagate_change(&mut self, node_idx: NodeIndex) {
        let to_compute = {
            self.audio_graph
                .lock()
                .unwrap()
                .collect_path_to_root(node_idx)
        };

        for (node, parent, edge) in to_compute {
            self.compute(node, parent, edge);

            self.audio_graph
                .lock()
                .unwrap()
                .get_node_mut(node)
                .unwrap()
                .process(); // Wrong but we'll see
        }
    }

    pub fn compute(&self, node: NodeIndex, parent: NodeIndex, edge: AudioGraphEdge) {
        let mut audio_graph = self.audio_graph.lock().unwrap();

        self.apply_effect(&mut audio_graph, node, parent, edge);
    }

    pub fn apply_effect(
        &self,
        graph: &mut AudioGraph,
        node_idx: NodeIndex,
        parent_idx: NodeIndex,
        effect: AudioGraphEdge,
    ) {
        let parent_node = graph
            .get_clip_mut(parent_idx)
            .expect("Parent node not found");

        let target_node = graph.get_clip_mut(node_idx).expect("Target node not found");

        match effect {
            AudioGraphEdge::Add => {}
            // ... other effects ...
            _ => {}
        }
    }

    pub fn get_root_sample(&mut self) -> Option<AudioFrame> {
        let audio_graph = self.audio_graph.lock().unwrap();
        let root_node = audio_graph.get_node_ref(self.root_node_index)?;

        match root_node.get_clip_ref() {
            AudioClipEnum::Mono(audio_clip) => {
                let frame = AudioClipTrait::get_frame(audio_clip, self.root_frame_idx);
                self.root_frame_idx += 1;
                Some(AudioFrame::Mono(frame))
            }
            AudioClipEnum::Stereo(audio_clip) => {
                let frame = AudioClipTrait::get_frame(audio_clip, self.root_frame_idx);
                self.root_frame_idx += 1;
                Some(AudioFrame::Stereo(frame))
            }
            _ => None,
        }
    }
}
