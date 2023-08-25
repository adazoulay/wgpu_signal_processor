// ----  Computation Tree ----

use super::audio_clip::{AudioClip, AudioClipTrait};
use super::audio_edge::{AddOperation, AudioGraphEdge};
use super::audio_node::AudioNode;
use petgraph::dot::Dot;
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableDiGraph};
use petgraph::visit::{Dfs, EdgeRef};
use std::collections::HashMap;
use std::fmt;

pub enum AudioGraphNode<F> {
    RootNode(AudioNode<F>),
    DataNode(AudioNode<F>),
}

impl<F> fmt::Display for AudioGraphNode<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioGraphNode::DataNode(node) => write!(f, "{}", node.name),
            AudioGraphNode::RootNode(node) => write!(f, "{}", node.name),
        }
    }
}

pub struct AudioGraph<F> {
    pub graph: StableDiGraph<AudioGraphNode<F>, AudioGraphEdge<F>>,
    pub root: NodeIndex,
    node_lookup: HashMap<String, NodeIndex>,
    node_id: i32,
}

impl<F> AudioGraph<F>
where
    F: dasp::Frame<Sample = f32> + Default + Copy,
{
    pub fn new() -> Self {
        let mut graph = StableDiGraph::new();
        let node_lookup = HashMap::new();
        let clip = AudioClip::<F>::default();
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
        //Todo  Add node_id in case of no name
        let name = audio_node.get_name().to_string();
        let node_id = self.graph.add_node(AudioGraphNode::DataNode(audio_node));
        self.node_lookup.insert(name, node_id);
        node_id
    }

    pub fn create_indexed_node(&mut self) -> AudioNode<F> {
        let name = &self.node_id.to_string();
        self.node_id += 1;
        let clip = AudioClip::<F>::default();
        AudioNode::new(clip, Some(name))
    }

    pub fn connect(
        &mut self,
        parent: NodeIndex,
        child: Option<NodeIndex>,
        edge: AudioGraphEdge<F>,
    ) -> Result<EdgeIndex, String> {
        let edge_id = match child {
            Some(child) => self.graph.add_edge(parent, child, edge),
            None => self.graph.add_edge(parent, self.root, edge),
        };
        Ok(edge_id)
    }

    pub fn collect_dependents(
        &self,
        node_idx: NodeIndex,
    ) -> Vec<(NodeIndex, NodeIndex, EdgeIndex)> {
        let mut dfs = Dfs::new(&self.graph, node_idx);
        let mut to_compute = Vec::new();

        while let Some(node) = dfs.next(&self.graph) {
            let incoming_info: Vec<(NodeIndex, NodeIndex, EdgeIndex)> = self
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .map(|edge| (node, edge.target(), edge.id()))
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

    pub fn get_edge_ref(&self, edge_idx: EdgeIndex) -> Option<&AudioGraphEdge<F>> {
        Some(&self.graph[edge_idx])
    }
    pub fn get_edge_mut(&mut self, edge_idx: EdgeIndex) -> Option<&AudioGraphEdge<F>> {
        Some(&mut self.graph[edge_idx])
    }

    pub fn print_graph(&self) {
        println!("{}", Dot::new(&self.graph));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dasp::frame::{Mono, Stereo};

    fn setup_graph<F: dasp::Frame<Sample = f32> + Default + Copy>() -> AudioGraph<F> {
        let mut graph = AudioGraph::<F>::new();

        let clip1 = AudioClip::<F>::default();
        let node1 = AudioNode::new(clip1, Some("node1"));
        graph.add_data_node(node1);

        let clip2 = AudioClip::<F>::default();
        let node2 = AudioNode::new(clip2, Some("node2"));
        graph.add_data_node(node2);

        let clip3 = AudioClip::<F>::default();
        let node3 = AudioNode::new(clip3, Some("node3"));
        graph.add_data_node(node3);

        let clip4 = AudioClip::<F>::default();
        let node4 = AudioNode::new(clip4, Some("node4"));
        graph.add_data_node(node4);

        let clip5 = AudioClip::<F>::default();
        let node5 = AudioNode::new(clip5, Some("node5"));
        graph.add_data_node(node5);

        let clip6 = AudioClip::<F>::default();
        let node6 = AudioNode::new(clip6, Some("node6"));
        graph.add_data_node(node6);

        let clip7 = AudioClip::<F>::default();
        let node7 = AudioNode::new(clip7, Some("node7"));
        graph.add_data_node(node7);

        let clip8 = AudioClip::<F>::default();
        let node8 = AudioNode::new(clip8, Some("node8"));
        graph.add_data_node(node8);

        let clip9 = AudioClip::<F>::default();
        let node9 = AudioNode::new(clip9, Some("node9"));
        graph.add_data_node(node9);

        graph
    }

    #[test]
    fn test_add_data_node_mono() {
        let mut graph = setup_graph::<Mono<f32>>();

        let clip = AudioClip::<Mono<f32>>::default();
        let node = AudioNode::new(clip, Some("test_node"));

        let index = graph.add_data_node(node);
        assert!(graph.get_node_ref(index).is_some());
        assert_eq!(graph.get_node_ref(index).unwrap().get_name(), "test_node");
    }

    #[test]
    fn test_add_data_node_stereo() {
        let mut graph = setup_graph::<Stereo<f32>>();

        let clip = AudioClip::<Stereo<f32>>::default();
        let node = AudioNode::new(clip, Some("test_node"));

        let index = graph.add_data_node(node);
        assert!(graph.get_node_ref(index).is_some());
        assert_eq!(graph.get_node_ref(index).unwrap().get_name(), "test_node");
    }

    #[test]
    fn test_connect_and_collect_path_mono() {
        let mut graph = setup_graph::<Mono<f32>>();

        let node1_id = graph.get_node_id("node1").unwrap();
        let node2_id = graph.get_node_id("node2").unwrap();
        let node3_id = graph.get_node_id("node3").unwrap();

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        let edge1 = graph.connect(node1_id, Some(node2_id), add_edge).unwrap();

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        let edge_2 = graph.connect(node2_id, Some(node3_id), add_edge).unwrap();

        let add_edge = AudioGraphEdge::new(AddOperation, "AddOp");
        let edge_3 = graph.connect(node3_id, None, add_edge).unwrap();

        graph.print_graph();

        let path = graph.collect_dependents(node1_id);
        // println!("Collected path: {:?}", path);

        assert_eq!(path.len(), 3);

        assert_eq!(path[0], (node1_id, node2_id, edge1));
        assert_eq!(path[1], (node2_id, node3_id, edge_2));
        assert_eq!(path[2], (node3_id, graph.root, edge_3));
    }

    #[test]
    fn test_connect_and_collect_path_stereo() {
        let mut graph = setup_graph::<Stereo<f32>>();

        let node1_id = graph.get_node_id("node1").unwrap();
        let node2_id = graph.get_node_id("node2").unwrap();
        let node3_id = graph.get_node_id("node3").unwrap();

        let edge1 = graph
            .connect(
                node1_id,
                Some(node2_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge2 = graph
            .connect(
                node2_id,
                Some(node3_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge3 = graph
            .connect(node3_id, None, AudioGraphEdge::new(AddOperation, "AddOp"))
            .unwrap();

        let path = graph.collect_dependents(node1_id);
        println!("Collected path: {:?}", path);

        assert_eq!(path.len(), 3);

        assert_eq!(path[0], (node1_id, node2_id, edge1));
        assert_eq!(path[1], (node2_id, node3_id, edge2));
        assert_eq!(path[2], (node3_id, graph.root, edge3));
    }

    #[test]
    fn test_connect_and_collect_path_mono_complex_1() {
        let mut graph = setup_graph::<Mono<f32>>();

        let node1_id = graph.get_node_id("node1").unwrap();
        let node2_id = graph.get_node_id("node2").unwrap();
        let node3_id = graph.get_node_id("node3").unwrap();
        let node4_id = graph.get_node_id("node4").unwrap();
        let node5_id = graph.get_node_id("node5").unwrap();
        let node6_id = graph.get_node_id("node6").unwrap();
        let node7_id = graph.get_node_id("node7").unwrap();

        let edge1 = graph
            .connect(
                node1_id,
                Some(node3_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge2 = graph
            .connect(
                node2_id,
                Some(node3_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge3 = graph
            .connect(
                node3_id,
                Some(node4_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge4 = graph
            .connect(
                node5_id,
                Some(node4_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge5 = graph
            .connect(
                node6_id,
                Some(node5_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge6 = graph
            .connect(
                node7_id,
                Some(node5_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge7 = graph
            .connect(node4_id, None, AudioGraphEdge::new(AddOperation, "AddOp"))
            .unwrap();

        graph.print_graph();

        let path = graph.collect_dependents(node1_id);
        println!("Collected path: {:?}", path);

        assert_eq!(path.len(), 3);

        assert_eq!(path[0], (node1_id, node3_id, edge1));
        assert_eq!(path[1], (node3_id, node4_id, edge3));
        assert_eq!(path[2], (node4_id, graph.root, edge7));
    }

    #[test]
    fn test_connect_and_collect_path_mono_complex_2() {
        let mut graph = setup_graph::<Mono<f32>>();

        let node1_id = graph.get_node_id("node1").unwrap();
        let node2_id = graph.get_node_id("node2").unwrap();
        let node3_id = graph.get_node_id("node3").unwrap();
        let node4_id = graph.get_node_id("node4").unwrap();
        let node5_id = graph.get_node_id("node5").unwrap();
        let node6_id = graph.get_node_id("node6").unwrap();
        let node7_id = graph.get_node_id("node7").unwrap();
        let node8_id = graph.get_node_id("node8").unwrap();
        let node9_id = graph.get_node_id("node9").unwrap();

        let edge1 = graph
            .connect(
                node1_id,
                Some(node4_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge2 = graph
            .connect(
                node2_id,
                Some(node4_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge3 = graph
            .connect(
                node2_id,
                Some(node5_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge4 = graph
            .connect(
                node3_id,
                Some(node5_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge5 = graph
            .connect(
                node7_id,
                Some(node5_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge6 = graph
            .connect(
                node7_id,
                Some(node6_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge7 = graph
            .connect(
                node3_id,
                Some(node9_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge8 = graph
            .connect(
                node1_id,
                Some(node8_id),
                AudioGraphEdge::new(AddOperation, "AddOp"),
            )
            .unwrap();

        let edge9 = graph
            .connect(node4_id, None, AudioGraphEdge::new(AddOperation, "AddOp"))
            .unwrap();

        let edge10 = graph
            .connect(node5_id, None, AudioGraphEdge::new(AddOperation, "AddOp"))
            .unwrap();

        let edge11 = graph
            .connect(node6_id, None, AudioGraphEdge::new(AddOperation, "AddOp"))
            .unwrap();

        graph.print_graph();

        let path = graph.collect_dependents(node1_id);
        println!("Collected path: {:?}", path);

        assert_eq!(path.len(), 3);
        assert_eq!(path[0], (node1_id, node8_id, edge8));
        assert_eq!(path[1], (node1_id, node4_id, edge1));
        assert_eq!(path[2], (node4_id, graph.root, edge9));
    }
}
