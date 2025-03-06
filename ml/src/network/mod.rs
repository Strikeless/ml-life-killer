use std::iter;

use itertools::Itertools;
use layer::{LayerOutputMap, NetworkLayer, NodeKey, input::InputLayer};
use serde::{Deserialize, Serialize};

pub mod layer;
pub mod node;

pub type Value = f32;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Network<S> {
    #[serde(skip)]
    pub input_layer: InputLayer<S>,

    pub mid_layers: Vec<NetworkLayer>,
    pub output_layer: NetworkLayer,
}

impl<S> Network<S> {
    pub fn new(
        input_layer: InputLayer<S>,
        mid_layer_count: usize,
        mid_layer_height: usize,
        output_layer_height: usize,
    ) -> Self {
        let mid_layers = iter::repeat_with(|| NetworkLayer::default_n_nodes(mid_layer_height))
            .take(mid_layer_count)
            .collect_vec();

        let output_layer = NetworkLayer::default_n_nodes(output_layer_height);

        Self {
            input_layer,
            mid_layers,
            output_layer,
        }
    }

    pub fn compute(&self, state: &S) -> LayerOutputMap {
        let inputs = self.input_layer.get_values(state);

        let compute_layers = self.mid_layers.iter().chain(iter::once(&self.output_layer));

        compute_layers.fold(inputs, |acc, layer| layer.compute(acc))
    }

    pub fn compute_to_iter(&self, state: &S) -> impl Iterator<Item = Value> {
        let outputs = self.compute(state);

        /*
        // Definitely same iteration order achieving SLOW HACK!
        let mut output_kv_pairs = outputs.into_iter().collect_vec();
        output_kv_pairs.sort_by_cached_key(|(node_key, _)| {
            let mut hasher = DefaultHasher::new();
            node_key.hash(&mut hasher);
            hasher.finish()
        });
        output_kv_pairs.into_iter().map(|(_, value)| value)
        */

        // NOTE: The outputs seem to always have the same iteration order.
        //       I don't think this really is specified anywhere though,
        //       so here might be the problem if nothing's working.
        outputs.into_iter().map(|(_, value)| value)
    }

    pub fn net_layers(&self) -> impl Iterator<Item = &NetworkLayer> {
        self.mid_layers.iter().chain(iter::once(&self.output_layer))
    }

    pub fn net_layers_mut(&mut self) -> impl Iterator<Item = &mut NetworkLayer> {
        self.mid_layers
            .iter_mut()
            .chain(iter::once(&mut self.output_layer))
    }

    pub fn net_layer(&self, index: usize) -> Option<&NetworkLayer> {
        self.net_layers().nth(index)
    }

    pub fn net_layer_mut(&mut self, index: usize) -> Option<&mut NetworkLayer> {
        self.net_layers_mut().nth(index)
    }

    pub fn iter_node_keys_by_layer(&self) -> impl Iterator<Item = Vec<NodeKey>> {
        let input_layer_keys = iter::once(self.input_layer.input_providers.keys().collect_vec());

        let net_layer_keys = self
            .net_layers()
            .map(|net_layer| net_layer.nodes.keys().collect_vec());

        input_layer_keys.chain(net_layer_keys)
    }
}
