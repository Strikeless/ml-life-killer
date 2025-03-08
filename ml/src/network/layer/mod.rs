use slotmap::{SecondaryMap, new_key_type};

use super::Value;

pub mod compute;
pub mod input;

new_key_type! {
    pub struct NodeKey;
}

pub type LayerOutputMap = SecondaryMap<NodeKey, Value>;

pub trait Layer {
    fn get_outputs(&self, inputs: Option<LayerOutputMap>) -> LayerOutputMap;

    // A vtable can't be built with an "impl Iterator" return type :(
    fn output_keys(&self) -> Vec<NodeKey>;
}
