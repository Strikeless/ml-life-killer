use slotmap::SlotMap;

use crate::network::Value;

use super::{LayerOutputMap, NodeKey};

pub type InputProvider<S> = fn(usize, &S) -> Value; // HACK: Provider index is just a hack for stupid reasons, please remove and figure out.

#[derive(Debug, Clone)]
pub struct InputLayer<S> {
    pub input_providers: SlotMap<NodeKey, InputProvider<S>>,
}

impl<S> InputLayer<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_input(mut self, provider: InputProvider<S>) -> Self {
        self.add_input(provider);
        self
    }

    pub fn add_input(&mut self, provider: InputProvider<S>) {
        self.input_providers.insert(provider);
    }

    pub fn get_values(&self, state: &S) -> LayerOutputMap {
        self.input_providers
            .iter()
            .enumerate()
            .map(|(provider_index, (key, provider))| (key, provider(provider_index, state)))
            .collect()
    }
}

impl<S> Default for InputLayer<S> {
    fn default() -> Self {
        Self {
            input_providers: SlotMap::with_key(),
        }
    }
}
