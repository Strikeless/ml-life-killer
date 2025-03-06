use std::iter;

use itertools::Itertools;
use libgame::{
    Game,
    board::{GameBoard, TileState},
    rule::Rule,
};
use rand::{
    Rng,
    seq::{IndexedMutRandom, IndexedRandom, IteratorRandom},
};

use crate::{
    network::{
        Network,
        layer::NodeKey,
        node::{Node, NodeInput},
    },
    player::{self, Player, kernel::Kernel},
};

pub struct Trainer {
    /// The number of independently mutated networks that contend in a single generation.
    pub generation_contenders: usize,

    /// The amount of mutations to apply to contenders in a single generation.
    pub generation_mutations: usize,

    /// The number of iterations in a single generation to average scores from.
    pub generation_iterations: usize,

    /// Whether or not the original network can be replaced by mutated derivatives, even if there was no score gain.
    pub generation_unstable: bool,

    /// The maximum amount of game steps in a single iteration run that can be played before giving up on the task.
    pub iteration_max_steps: usize,

    pub game_board_width: usize,
    pub game_board_height: usize,
    pub game_board_alive_cells: usize,

    pub player_network_consecutive_turns: usize,
    pub player_game_consecutive_turns: usize,
}

// Temporary thingy for network state
type S = Kernel<1>;

impl Trainer {
    /// A generation consists of a single set of mutated networks based on the previous network,
    /// of which the best average-performers are selected.
    pub fn train_generation(&self, network: Network<S>) -> (Network<S>, isize)
    where
        S: Clone,
    {
        // -1 because we chain the original network.
        let contenders = iter::repeat_n(network.clone(), self.generation_contenders - 1)
            .map(|mut new_contender| {
                for _ in 0..self.generation_mutations {
                    new_contender = self.mutate(new_contender);
                }

                new_contender
            })
            .chain(iter::once(network));

        let mut scoring_contenders = contenders
            .map(|contender| (contender, Vec::new()))
            .collect_vec();

        for _ in 0..self.generation_iterations {
            let iteration = TrainingIteration::new_random(
                self.game_board_width,
                self.game_board_height,
                self.game_board_alive_cells,
                self.iteration_max_steps,
                self.player_network_consecutive_turns,
                self.player_game_consecutive_turns,
            );

            for (network, scores) in &mut scoring_contenders {
                let performance = iteration.try_out(&network);
                scores.push(performance);
            }
        }

        let scored_contenders = scoring_contenders.into_iter().map(|(contender, scores)| {
            let score_count = scores.len();

            // HACK: Times ten to prevent precision loss leading to worse evolution.
            let total_score = (scores.into_iter().sum::<isize>() * 10) / score_count as isize;

            (contender, total_score)
        });

        // SAFETY: There's always going to be atleast one contender, so this shouldn't ever be an issue.
        // TODO: Implement unstable again.
        scored_contenders.max_by_key(|(_, score)| *score).unwrap()
    }

    fn mutate(&self, mut network: Network<S>) -> Network<S> {
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

        fn weight_adjustment(network: &mut Network<S>) -> Option<Mutation> {
            let rng = &mut rand::rng();

            let net_layer = network.net_layers_mut().choose(rng)?;
            let node = net_layer.nodes.values_mut().choose(rng)?;
            let input = node.inputs.choose_mut(rng)?;

            let adjustment_max_magnitude = (input.weight.abs() / 2.0).max(0.01);
            let adjustment = rng.random_range(-adjustment_max_magnitude..adjustment_max_magnitude);

            Some(Mutation::AdjustWeight { input, adjustment })
        }

        fn input_creation(network: &mut Network<S>) -> Option<Mutation> {
            let rng = &mut rand::rng();

            let net_layer_count = network.net_layers().count();
            let net_layer_index =
                (net_layer_count > 0).then(|| rng.random_range(0..net_layer_count))?;

            let src_node_key = {
                // NOTE: Net layer index is already the previous layer index here since it doesn't include the input layer!
                let prev_layer_node_keys =
                    network.iter_node_keys_by_layer().nth(net_layer_index)?;

                prev_layer_node_keys.into_iter().choose(rng)?
            };

            let node = {
                let layer = network.net_layers_mut().nth(net_layer_index)?;
                layer.nodes.values_mut().choose(rng)?
            };

            if node
                .inputs
                .iter()
                .any(|input| input.node_key == src_node_key)
            {
                return None;
            }

            let weight = rng.random_range(-1.0..1.0);

            Some(Mutation::InputCreation {
                node,
                src_node_key,
                weight,
            })
        }

        fn input_deletion(network: &mut Network<S>) -> Option<Mutation> {
            let rng = &mut rand::rng();

            let net_layer = network.net_layers_mut().choose(rng)?;
            let node = net_layer.nodes.values_mut().choose(rng)?;

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

struct TrainingIteration {
    game: Game,
    max_steps: usize,
    player_network_consecutive_turns: usize,
    player_game_consecutive_turns: usize,
}

impl TrainingIteration {
    pub fn new_random(
        width: usize,
        height: usize,
        alive_cells: usize,
        max_steps: usize,
        player_network_consecutive_turns: usize,
        player_game_consecutive_turns: usize,
    ) -> Self {
        let board = GameBoard::new_random(width, height, alive_cells);
        let game = Game::new(board, Rule::default());

        Self {
            game,
            max_steps,
            player_network_consecutive_turns,
            player_game_consecutive_turns,
        }
    }

    // TODO: How the hell are we gonna deal with multi-network players?
    pub fn try_out(&self, network: &Network<Kernel<1>>) -> isize {
        let mut player = Player {
            // FIXME: This shouldn't need yet another network clone.
            network_1x1: network.clone(),
            game: self.game.clone(),

            network_consecutive_turns: self.player_network_consecutive_turns,
            game_consecutive_turns: self.player_game_consecutive_turns,
        };

        let initial_alive_cells = Self::count_alive_cells(&player);

        for step in 0..self.max_steps {
            player.play_step();

            if Self::count_alive_cells(&player) == 0 {
                // Task accomplished, reward the least steps taken.
                return (self.max_steps - step) as isize;
            }
        }

        // Task wasn't accomplished, punish the least cells killed.
        let end_alive_cells = Self::count_alive_cells(&player);
        -100 + initial_alive_cells as isize - end_alive_cells as isize
    }

    fn count_alive_cells(player: &Player) -> usize {
        player
            .game
            .board
            .tiles
            .iter()
            .filter(|tile| **tile == TileState::Alive)
            .count()
    }
}
