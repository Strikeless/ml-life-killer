use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

use crate::network::{node::Node, NetworkConfig};

use super::{Layer, LayerOutputMap, NodeKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeLayer {
    pub nodes: SlotMap<NodeKey, Node>,
}

impl ComputeLayer {
    pub fn default_n_nodes(count: usize) -> Self {
        let mut nodes = SlotMap::with_capacity_and_key(count);
        for _ in 0..count {
            nodes.insert(Node::default());
        }

        Self { nodes }
    }
}

impl Layer for ComputeLayer {
    fn get_outputs(&self, config: &NetworkConfig, inputs: Option<LayerOutputMap>) -> LayerOutputMap {
        let inputs = inputs.expect("Compute layer wasn't given inputs");

        self.nodes
            .iter()
            .map(|(key, node)| {
                let value = node.compute(config, &inputs);
                (key, value)
            })
            .collect()
    }

    fn output_keys(&self) -> Vec<NodeKey> {
        self.nodes.keys().collect()
    }
}
