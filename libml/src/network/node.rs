use serde::{Deserialize, Serialize};

use super::{
    NetworkConfig, Value
};

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Node {
    pub inputs: Vec<NodeInput>,
}

impl Node {
    pub fn compute(&self, config: &NetworkConfig, input_values: &Vec<Value>) -> Value {
        self.inputs
            .iter()
            .map(|input| input.compute(config, input_values))
            .reduce(|a, b| config.combinator.combine(a, b))
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct NodeInput {
    pub node_index: usize,
    pub weight: Value,
}

impl NodeInput {
    fn compute(&self, config: &NetworkConfig, input_values: &Vec<Value>) -> Value {
        let input_value = input_values.get(self.node_index).expect("Missing input");
        let weighted_value = input_value * self.weight;
        config.activator.activate(weighted_value)
    }
}
