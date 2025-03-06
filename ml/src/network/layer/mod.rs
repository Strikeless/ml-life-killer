pub mod input;

use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SlotMap, new_key_type};

use super::{Value, node::Node};

new_key_type! {
    pub struct NodeKey;
}

pub type LayerOutputMap = SecondaryMap<NodeKey, Value>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkLayer {
    pub nodes: SlotMap<NodeKey, Node>,
}

impl NetworkLayer {
    pub fn default_n_nodes(count: usize) -> Self {
        let mut nodes = SlotMap::with_capacity_and_key(count);
        for _ in 0..count {
            nodes.insert(Node::default());
        }

        Self { nodes }
    }

    pub fn compute(&self, input_values: LayerOutputMap) -> LayerOutputMap {
        self.nodes
            .iter()
            .map(|(key, node)| {
                let value = node.compute(&input_values);
                (key, value)
            })
            .collect()
    }
}
