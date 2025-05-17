use serde::{Deserialize, Serialize};

use crate::network::{NetworkConfig, Value};

use super::Layer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputLayer {
    height: usize,

    // FIXME: It doesn't make sense that we're storing the values inside the network.
    //        This is the only reason adapters need a mutable reference to the network.
    //        We would still need the keys to be stable between serialization cycles, but not the last used values.
    output_values: Vec<Value>,
}

impl InputLayer {
    pub fn new(height: usize) -> Self {
        let output_values = vec![0.0; height];

        Self {
            height,
            output_values,
        }
    }

    pub fn update<'a, I>(&mut self, values: I)
    where
        I: IntoIterator<Item = Value>,
    {
        let mut value_iter = values.into_iter();
        let mut target_value_iter = self.output_values.iter_mut();

        while let Some(value) = value_iter.next() {
            let Some(target_value) = target_value_iter.next() else {
                panic!("Input layer too short ({}) for all values", self.output_values.len());
            };

            *target_value = value;
        }
    }
}

impl Layer for InputLayer {
    fn get_outputs(&self, _config: &NetworkConfig, _inputs: Option<Vec<Value>>) -> Vec<Value> {
        self.output_values.clone()
    }

    fn output_node_indices(&self) -> Vec<usize> {
        (0..self.output_values.len())
            .collect()
    }
}
