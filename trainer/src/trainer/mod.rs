use std::{iter, marker::PhantomData};

use itertools::Itertools;
use libml::network::{
    Network,
    layer::NodeKey,
    node::{Node, NodeInput},
};
use rand::{
    Rng,
    seq::{IndexedMutRandom, IndexedRandom, IteratorRandom},
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::adapter::{TrainerAdapter, TrainerAdapterFactory};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct TrainerConfig {
    /// The number of independently mutated networks that contend in a single generation.
    pub generation_contenders: usize,

    /// The amount of mutations to apply to contenders in a single generation.
    pub generation_mutations: usize,

    /// The amount of randomization to add to the mutation count of each competing contender in a generation.
    pub generation_mutations_jitter: usize,

    /// The number of iterations in a single generation to average scores from.
    pub generation_iterations: usize,
}

pub struct Trainer<A, AF>
where
    A: TrainerAdapter,
    AF: TrainerAdapterFactory<A>,
{
    pub config: TrainerConfig,

    pub adapter_factory: AF,
    _adapter_phantom: PhantomData<A>,
}

impl<A, AF> Trainer<A, AF>
where
    A: TrainerAdapter,
    AF: TrainerAdapterFactory<A>,
{
    pub fn new(config: TrainerConfig, adapter_factory: AF) -> Self {
        Self {
            config,
            adapter_factory,
            _adapter_phantom: PhantomData,
        }
    }
}

impl<A, AF> Trainer<A, AF>
where
    A: TrainerAdapter,
    AF: TrainerAdapterFactory<A>,
{
    /// A generation consists of a single set of mutated networks based on the previous network,
    /// of which the best average-performers are selected.
    pub fn train_generation(&self, network: Network) -> (Network, isize) {
        // NOTE: -1 because we chain the original network.
        let contenders = iter::repeat_n(network.clone(), self.config.generation_contenders - 1)
            .map(|mut new_contender| {
                let mutation_count_randomization = rand::rng().random_range(
                    -(self.config.generation_mutations_jitter as i32)..(self.config.generation_mutations_jitter as i32)
                );

                let mutation_count = (self.config.generation_mutations as i32 + mutation_count_randomization).max(1) as usize;
                for _ in 0..mutation_count {
                    new_contender = self.mutate(new_contender);
                }

                new_contender
            })
            .chain(iter::once(network));

        let mut scoring_contenders = contenders
            .map(|contender| (contender, Vec::new()))
            .collect_vec();

        for _ in 0..self.config.generation_iterations {
            let iteration_adapter = self.adapter_factory.create_adapter();

            // Hide performance issues under the mat with the power of multithreading!
            // PERF: Would it be possible to parallelize this further by doing many iterations simultaneously?
            scoring_contenders
                .par_iter_mut()
                .for_each(|(network, scores)| {
                    let performance = iteration_adapter.try_out(network);
                    scores.push(performance);
                });
        }

        let scored_contenders = scoring_contenders.into_iter().map(|(contender, scores)| {
            let score_count = scores.len();

            // HACK: Times ten to avoid precision loss of average score due to the integer conversion.
            let total_score = scores.into_iter().sum::<isize>() * 10 / score_count as isize;

            (contender, total_score)
        });

        // SAFETY: There's always going to be atleast one contender (due to including the original network),
        //         so unwrap should always be OK.
        scored_contenders.max_by_key(|(_, score)| *score).unwrap()
    }

    fn mutate(&self, mut network: Network) -> Network {
        #[derive(Debug)]
        enum Mutation<'a> {
            AdjustWeight {
                input: &'a mut NodeInput,
                adjustment: f32,
            },
            InputCreation {
                node: &'a mut Node,
                src_node_key: NodeKey,
                weight: f32,
            },
            InputDeletion {
                node: &'a mut Node,
                input_index: usize,
            },
        }

        fn weight_adjustment(network: &mut Network) -> Option<Mutation> {
            let rng = &mut rand::rng();

            let comp_layer = network.compute_layers.choose_mut(rng)?;
            let node = comp_layer.nodes.values_mut().choose(rng)?;
            let input = node.inputs.choose_mut(rng)?;

            let adjustment_max_magnitude = (input.weight.abs() / 2.0).max(0.01);
            let adjustment = rng.random_range(-adjustment_max_magnitude..adjustment_max_magnitude);

            Some(Mutation::AdjustWeight { input, adjustment })
        }

        fn input_creation(network: &mut Network) -> Option<Mutation> {
            let rng = &mut rand::rng();

            let comp_layer_count = network.compute_layers.len();
            let comp_layer_index =
                (comp_layer_count > 0).then(|| rng.random_range(0..comp_layer_count))?;

            let src_node_key = {
                // NOTE: comp_layer_index is already the previous layer index here since we're going
                //       from a compute layer index (doesn't include input layer!) to a layer index.
                let prev_layer_index = comp_layer_index;
                let prev_layer_node_keys = network.layer(prev_layer_index)?.output_keys();
                prev_layer_node_keys.into_iter().choose(rng)?
            };

            let node = {
                let layer = network.compute_layers.get_mut(comp_layer_index)?;
                layer.nodes.values_mut().choose(rng)?
            };

            // If the selected node already has an input to the same source node, don't continue.
            if node
                .inputs
                .iter()
                .any(|input| input.node_key == src_node_key)
            {
                return None;
            }

            let weight = rng.random_range(-2.0..=2.0);

            Some(Mutation::InputCreation {
                node,
                src_node_key,
                weight,
            })
        }

        fn input_deletion(network: &mut Network) -> Option<Mutation> {
            let rng = &mut rand::rng();

            let comp_layer = network.compute_layers.choose_mut(rng)?;
            let node = comp_layer.nodes.values_mut().choose(rng)?;

            let input_index = {
                let input_count = node.inputs.len();
                (input_count > 0).then(|| rng.random_range(0..input_count))?
            };

            Some(Mutation::InputDeletion { node, input_index })
        }

        let preferred_mutation_providers = {
            let preferred_mutation_type_choice = [(0, 7), (1, 1), (2, 2)]
                .choose_weighted(&mut rand::rng(), |(_, weight)| *weight)
                .map(|(choice, _)| *choice)
                .unwrap();

            match preferred_mutation_type_choice {
                0 => vec![weight_adjustment, input_creation, input_deletion],
                1 => vec![input_creation, weight_adjustment, input_deletion],
                2 => vec![input_deletion, weight_adjustment, input_creation],
                _ => unreachable!(),
            }
        };

        for mutation_provider in preferred_mutation_providers.into_iter() {
            let Some(mutation) = mutation_provider(&mut network) else {
                continue;
            };

            match mutation {
                Mutation::AdjustWeight { input, adjustment } => {
                    input.weight += adjustment;
                }
                Mutation::InputCreation {
                    node,
                    src_node_key,
                    weight,
                } => {
                    node.inputs.push(NodeInput {
                        node_key: src_node_key,
                        weight,
                    });
                }
                Mutation::InputDeletion { node, input_index } => {
                    node.inputs.remove(input_index);
                }
            }
        }

        network
    }
}
