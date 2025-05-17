use serde::{Deserialize, Serialize};

use crate::network::{node::Node, NetworkConfig, Value};

use super::Layer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeLayer {
    pub nodes: Vec<Node>,
}

impl ComputeLayer {
    pub fn default_n_nodes(count: usize) -> Self {
        let nodes = vec![Node::default(); count];
        Self { nodes }
    }
}

impl Layer for ComputeLayer {
    fn get_outputs(&self, config: &NetworkConfig, inputs: Option<Vec<Value>>) -> Vec<Value> {
        let inputs = inputs.expect("Compute layer wasn't given inputs");

        self.nodes
            .iter()
            .map(|node| node.compute(config, &inputs))
            .collect()
    }

    fn output_node_indices(&self) -> Vec<usize> {
        (0..self.nodes.len())
            .collect()
    }
}
