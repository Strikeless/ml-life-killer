use serde::{Deserialize, Serialize};

use super::{
    Value,
    layer::{LayerOutputMap, NodeKey},
};

use std::ops::Add;

const VALUE_COMBINATOR: fn(Value, Value) -> Value = f32::add;
const ACTIVATION_FUNCTION: fn(Value) -> Value = |input| /*input.sin()*/ input.tanh();

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Node {
    pub inputs: Vec<NodeInput>,
}

impl Node {
    pub fn compute(&self, input_values: &LayerOutputMap) -> Value {
        self.inputs
            .iter()
            .map(|input| input.compute(input_values))
            .fold(0.0, VALUE_COMBINATOR)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct NodeInput {
    pub node_key: NodeKey,
    pub weight: Value,
}

impl NodeInput {
    fn compute(&self, input_values: &LayerOutputMap) -> Value {
        let input_value = *input_values.get(self.node_key).expect("Missing input");
        let weighted_value = input_value * self.weight;
        ACTIVATION_FUNCTION(weighted_value)
    }
}
