use serde::{Deserialize, Serialize};

use super::{
    layer::{LayerOutputMap, NodeKey}, NetworkConfig, Value
};

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Node {
    pub inputs: Vec<NodeInput>,
}

impl Node {
    pub fn compute(&self, config: &NetworkConfig, input_values: &LayerOutputMap) -> Value {
        self.inputs
            .iter()
            .map(|input| input.compute(config, input_values))
            .fold(0.0, |a, b| config.combinator.combine(a, b))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct NodeInput {
    pub node_key: NodeKey,
    pub weight: Value,
}

impl NodeInput {
    fn compute(&self, config: &NetworkConfig, input_values: &LayerOutputMap) -> Value {
        let input_value = *input_values.get(self.node_key).expect("Missing input");
        let weighted_value = input_value * self.weight;
        config.activator.activate(weighted_value)
    }
}
