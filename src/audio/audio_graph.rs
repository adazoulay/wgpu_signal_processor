// ----  Computation Tree ----

use crate::audio::audio_clip::AudioClipEnum;
use crate::audio::audio_node::AudioNode;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::visit::{Dfs, EdgeRef};

use std::collections::hash_map::HashMap;

pub enum AudioGraphNode<F> {
    RootNode(AudioNode<F>),
    DataNode(AudioNode<F>),
}

#[derive(Copy, Clone)]
pub enum AudioGraphEdge {
    Add,       // Add clips to another node
    Crossfade, // Blend two audio signals with a crossfade.
    Subtract,  // Subtract one signal from another.
    Multiply,  // Multiply two audio signals (modulation).
    Bypass, // Pass the audio through without any alterations. Useful for optionally skipping nodes.
}

pub struct AudioGraph<F> {
    graph: StableDiGraph<AudioGraphNode<F>, AudioGraphEdge>,
    pub root: NodeIndex,
    node_lookup: HashMap<String, NodeIndex>,
    node_id: i32,
}

impl<F> AudioGraph<F> {
    pub fn new() -> Self {
        let mut graph = StableDiGraph::new();
        let node_lookup = HashMap::new();
        let clip = AudioClipEnum::default();
        let audio_node = AudioGraphNode::RootNode(AudioNode::new(clip, Some("root")));
        let root: petgraph::stable_graph::NodeIndex = graph.add_node(audio_node);
        Self {
            graph,
            node_lookup,
            root,
            node_id: 1,
        }
    }

    pub fn add_data_node(&mut self, audio_node: AudioNode<F>) -> NodeIndex {
        let name = audio_node.get_name().to_string();
        let node_id = self.graph.add_node(AudioGraphNode::DataNode(audio_node));
        self.node_lookup.insert(name, node_id);
        node_id
    }

    pub fn create_indexed_node(&mut self) -> AudioNode<F> {
        let name = &self.node_id.to_string();
        self.node_id += 1;
        let clip = AudioClipEnum::default();
        AudioNode::new(clip, Some(name))
    }

    pub fn connect(&mut self, n1: NodeIndex, n2: Option<NodeIndex>, op: AudioGraphEdge) {
        match n2 {
            Some(n2) => self.graph.add_edge(n1, n2, op),
            None => self.graph.add_edge(n1, self.root, op),
        };
    }

    pub fn collect_path_to_root(
        &self,
        node_idx: NodeIndex,
    ) -> Vec<(NodeIndex, NodeIndex, AudioGraphEdge)> {
        let mut dfs = Dfs::new(&self.graph, node_idx);
        let mut to_compute = Vec::new();

        while let Some(node) = dfs.next(&self.graph) {
            let incoming_info: Vec<(NodeIndex, NodeIndex, AudioGraphEdge)> = self
                .graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .map(|edge| (node, edge.source(), edge.weight().clone()))
                .collect::<Vec<_>>();

            to_compute.extend(incoming_info);
        }

        to_compute
    }

    pub fn get_node_id(&self, id: &str) -> Option<NodeIndex> {
        self.node_lookup.get(id).cloned()
    }

    pub fn get_node_mut(&mut self, node_idx: NodeIndex) -> Option<&mut AudioNode<F>> {
        match &mut self.graph[node_idx] {
            AudioGraphNode::RootNode(node) => Some(node),
            AudioGraphNode::DataNode(node) => Some(node),
        }
    }

    pub fn get_node_ref(&self, node_idx: NodeIndex) -> Option<&AudioNode<F>> {
        match &self.graph[node_idx] {
            AudioGraphNode::RootNode(node) => Some(node),
            AudioGraphNode::DataNode(node) => Some(node),
        }
    }

    pub fn get_clip_mut(&mut self, node_idx: NodeIndex) -> Option<&mut AudioClipEnum> {
        match &mut self.graph[node_idx] {
            AudioGraphNode::DataNode(node) => Some(node.get_clip_mut()),
            AudioGraphNode::RootNode(node) => Some(node.get_clip_mut()),
        }
    }
}
