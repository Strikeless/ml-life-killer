use super::{NetworkConfig, Value};

pub mod compute;
pub mod input;

pub trait Layer {
    fn get_outputs(&self, config: &NetworkConfig, inputs: Option<Vec<Value>>) -> Vec<Value>;

    // A vtable can't be built with an "impl Iterator" return type :(
    fn output_node_indices(&self) -> Vec<usize>;
}
