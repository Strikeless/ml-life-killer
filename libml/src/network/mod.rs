use std::iter;

use functions::{Activator, Combinator};
use itertools::Itertools;
use layer::{Layer, LayerOutputMap, compute::ComputeLayer, input::InputLayer};
use serde::{Deserialize, Serialize};

pub mod harness;
pub mod layer;
pub mod node;
pub mod functions;

pub type Value = f32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub activator: Activator,
    pub combinator: Combinator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub config: NetworkConfig,
    pub input_layer: InputLayer,
    pub compute_layers: Vec<ComputeLayer>,
}

impl Network {
    pub fn new(
        config: NetworkConfig,
        input_layer_height: usize,
        hidden_layer_count: usize,
        hidden_layer_height: usize,
        output_layer_height: usize,
    ) -> Self {
        let input_layer = InputLayer::new(input_layer_height);

        let compute_layers = {
            let hidden_layers =
                iter::repeat_with(|| ComputeLayer::default_n_nodes(hidden_layer_height))
                    .take(hidden_layer_count);

            let output_layer_iter = iter::once(ComputeLayer::default_n_nodes(output_layer_height));

            hidden_layers.chain(output_layer_iter).collect_vec()
        };

        Self {
            config,
            input_layer,
            compute_layers,
        }
    }

    pub fn compute(&self) -> LayerOutputMap {
        self.layers()
            .fold(None, |inputs, layer| Some(layer.get_outputs(&self.config, inputs)))
            .expect("No layers")
    }

    pub fn layers(&self) -> impl Iterator<Item = &dyn Layer> {
        iter::once(&self.input_layer as &dyn Layer).chain(
            self.compute_layers
                .iter()
                .map(|compute_layer| compute_layer as &dyn Layer),
        )
    }

    pub fn layer(&self, index: usize) -> Option<&dyn Layer> {
        self.layers().nth(index)
    }
}
