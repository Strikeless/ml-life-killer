use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

use crate::network::{NetworkConfig, Value};

use super::{Layer, NodeKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputLayer {
    height: usize,

    // FIXME: It doesn't make sense that we're storing the values inside the network.
    //        This is the only reason adapters need a mutable reference to the network.
    //        We would still need the keys to be stable between serialization cycles, but not the last used values.
    output_values: SlotMap<NodeKey, Value>,
}

impl InputLayer {
    pub fn new(height: usize) -> Self {
        let mut output_values = SlotMap::with_capacity_and_key(height);
        for _ in 0..height {
            output_values.insert(0.0);
        }

        Self {
            height,
            output_values,
        }
    }

    pub fn update<'a, I>(&mut self, values: I)
    where
        I: IntoIterator<Item = Value>,
    {
        let mut values = values.into_iter();

        self.output_values = SlotMap::with_capacity_and_key(self.height);
        for _ in 0..self.height {
            let value = values.next().expect("Missing input value");
            self.output_values.insert(value); // WARN: This is nonsense and we're lucky if this always results in the same keys.
        }

        debug_assert!(
            values.next().is_none(),
            "Input layer too short for all values ({} < {})",
            self.output_values.len(),
            values.count() + 1, // NOTE: +1 because we already consumed one in the assert condition.
        );
    }
}

impl Layer for InputLayer {
    fn get_outputs(&self, _config: &NetworkConfig, _inputs: Option<super::LayerOutputMap>) -> super::LayerOutputMap {
        self.output_values
            .iter()
            .map(|(key, value_ref)| (key, *value_ref))
            .collect()
    }

    fn output_keys(&self) -> Vec<NodeKey> {
        self.output_values.keys().collect()
    }
}
