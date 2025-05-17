use libml::network::{node::{Node, NodeInput}, Network};
use rand::{seq::{IndexedMutRandom, IteratorRandom}, Rng};

#[derive(Debug)]
pub enum Mutation<'a> {
    AdjustWeight {
        input: &'a mut NodeInput,
        adjustment: f32,
    },
    InputCreation {
        node: &'a mut Node,
        src_node_index: usize,
        weight: f32,
    },
    InputDeletion {
        node: &'a mut Node,
        input_index: usize,
    },
}

pub fn weight_adjustment(network: &mut Network) -> Option<Mutation> {
    let rng = &mut rand::rng();

    let comp_layer = network.compute_layers.choose_mut(rng)?;
    let node = comp_layer.nodes.iter_mut().choose(rng)?;
    let input = node.inputs.choose_mut(rng)?;

    let adjustment_max_magnitude = (input.weight.abs() / 2.0).max(0.01);
    let adjustment = rng.random_range(-adjustment_max_magnitude..adjustment_max_magnitude);

    Some(Mutation::AdjustWeight { input, adjustment })
}

pub fn input_creation(network: &mut Network) -> Option<Mutation> {
    let rng = &mut rand::rng();

    let comp_layer_count = network.compute_layers.len();
    let comp_layer_index =
        (comp_layer_count > 0).then(|| rng.random_range(0..comp_layer_count))?;

    let src_node_index = {
        // NOTE: comp_layer_index is already the previous layer index here since we're going
        //       from a compute layer index (doesn't include input layer!) to a general layer index.
        let prev_layer_index = comp_layer_index;
        let prev_layer_node_indices = network.layer(prev_layer_index)?.output_node_indices();
        prev_layer_node_indices.into_iter().choose(rng)?
    };

    let node = {
        let layer = network.compute_layers.get_mut(comp_layer_index)?;
        layer.nodes.iter_mut().choose(rng)?
    };

    // If the selected node already has an input to the same source node, don't continue.
    if node
        .inputs
        .iter()
        .any(|input| input.node_index == src_node_index)
    {
        return None;
    }

    let weight = rng.random_range(-2.0..=2.0);

    Some(Mutation::InputCreation {
        node,
        src_node_index,
        weight,
    })
}

pub fn input_deletion(network: &mut Network) -> Option<Mutation> {
    let rng = &mut rand::rng();

    let comp_layer = network.compute_layers.choose_mut(rng)?;
    let node = comp_layer.nodes.iter_mut().choose(rng)?;

    let input_index = {
        let input_count = node.inputs.len();
        (input_count > 0).then(|| rng.random_range(0..input_count))?
    };

    Some(Mutation::InputDeletion { node, input_index })
}
