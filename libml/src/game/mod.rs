use std::{cmp::Ordering, collections::HashMap};

use itertools::Itertools;
use kernel::Kernel;
use libgame::{Game, board::TileState, pos::Position};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::network::{Network, harness::NetworkHarness};

pub mod kernel;
pub mod networksave;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NetworkPlayerConfig {
    pub kernel_diameter: usize,

    /// Whether to reuse network responses for identical kernels. This renders any kind of randomization/heat quite useless
    /// but can make training significantly faster for bigger board or kernel sizes, trading memory usage for performance.
    pub use_kernel_cache: bool,
}

pub struct NetworkPlayer<'a> {
    pub config: NetworkPlayerConfig,
    pub network_harness: NetworkHarness<'a, Kernel>,
    kernel_cache: Option<HashMap<Kernel, KernelOutput>>,
}

pub struct NetworkPlayerMove {
    pub position: Position,
    pub new_state: TileState,
}

impl<'a> NetworkPlayer<'a> {
    pub fn new(config: NetworkPlayerConfig, network: &'a mut Network) -> Self {
        let network_harness = NetworkHarness::new(network)
            .with_inputs(Kernel::input_providers(config.kernel_diameter));

        Self {
            config,
            network_harness,
            kernel_cache: config.use_kernel_cache.then(|| HashMap::new()),
        }
    }

    pub fn play_step(&mut self, game: &mut Game) -> Option<NetworkPlayerMove> {
        if let Some((chosen_position, output)) = self.compute(&game) {
            // SAFETY: The compute method doesn't let the network give arbitrary positions,
            //         so positions will always correspond to a tile.
            let chosen_tile = game.board.tile_mut(chosen_position).unwrap();

            let wanted_state = match output.state {
                ..-0.5 => Some(TileState::Dead),
                0.5.. => Some(TileState::Alive),
                _ => None,
            };

            let current_state = *chosen_tile;
            if let Some(wanted_state) = wanted_state
                && wanted_state != current_state
            {
                *chosen_tile = wanted_state;

                Some(NetworkPlayerMove {
                    new_state: wanted_state,
                    position: chosen_position,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn compute(&mut self, game: &Game) -> Option<(Position, KernelOutput)> {
        // Cartesian_product is smartie speech for all the unique combinations of items.
        let positions = (0..game.board.width)
            .cartesian_product(0..game.board.height)
            .map(|(x, y)| Position { x, y });

        // Randomize the order of tiles as an attempt to force the network to be smarter about it's choices,
        // since now it can't just always say "yeah I want a cell here because the previous tile has one".
        let positions = {
            let mut positions_vec = positions.collect_vec();
            positions_vec.shuffle(&mut rand::rng());
            positions_vec.into_iter()
        };

        let scored_positions = positions
            .into_iter()
            .map(|pos| (pos, self.compute_pos(game, pos)));

        // Pick the top by highest score.
        scored_positions.into_iter().max_by(
            |(_, KernelOutput { score: a_score, .. }), (_, KernelOutput { score: b_score, .. })| {
                // Due to NaNs or something Rust doesn't provide Ord implementations for floats so gotta do with this.
                if a_score > b_score {
                    Ordering::Greater
                } else if a_score == b_score {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            },
        )
    }

    fn compute_pos(&mut self, game: &Game, pos: Position) -> KernelOutput {
        let kernel = self.get_kernel(game, pos);

        if let Some(kernel_cache) = &self.kernel_cache {
            if let Some(cached_output) = kernel_cache.get(&kernel) {
                return *cached_output;
            }
        }

        let mut network_outputs = self.network_harness.compute(&kernel);

        let mut next = || {
            network_outputs
                .next()
                .expect("Not enough outputs in kernel network")
        };

        let output = KernelOutput {
            score: next(),
            state: next(),
        };
        if let Some(kernel_cache) = &mut self.kernel_cache {
            drop(next);
            drop(network_outputs);
            kernel_cache.insert(kernel, output);
        }

        output
    }

    fn get_kernel(&self, game: &Game, center_pos: Position) -> Kernel {
        let kernel_radius = (self.config.kernel_diameter / 2) as isize;

        let relative_positions =
            (-kernel_radius..=kernel_radius).cartesian_product(-kernel_radius..=kernel_radius);

        let positions = relative_positions.map(|(rel_x, rel_y)| {
            Some(Position {
                x: (center_pos.x as isize)
                    .checked_add(rel_x)
                    .filter(|val| val.is_positive())? as usize,
                y: (center_pos.y as isize)
                    .checked_add(rel_y)
                    .filter(|val| val.is_positive())? as usize,
            })
        });

        let tiles = positions
            .map(|maybe_position| game.board.tile(maybe_position?).copied())
            .collect_vec();

        Kernel { tiles }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct KernelOutput {
    /// The score by which selecting the position should be preferred.
    pub score: f32,

    // A value representing how much the network wants the tile to be alive.
    pub state: f32,
}
