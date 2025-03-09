use std::cmp::Ordering;

use itertools::Itertools;
use libgame::{Game, board::TileState, pos::Position};

use crate::network::{Network, harness::NetworkHarness};

use super::{GameTrainerAdapterConfig, kernel::Kernel};

pub struct NetworkPlayer<'a> {
    pub config: GameTrainerAdapterConfig,
    pub network_harness: NetworkHarness<'a, Kernel>,
    pub game: Game,
}

impl<'a> NetworkPlayer<'a> {
    pub fn new(
        network: &'a mut Network,
        game_template: &Game,
        config: GameTrainerAdapterConfig,
    ) -> Self {
        let network_harness = NetworkHarness::new(network)
            .with_inputs(Kernel::input_providers(config.kernel_diameter));

        let game = game_template.clone();

        Self {
            config,
            network_harness,
            game,
        }
    }

    pub fn play_step(&mut self) {
        for _ in 0..self.config.network_consecutive_turns {
            self.network_make_move();
        }

        for _ in 0..self.config.game_consecutive_turns {
            self.game.tick();
        }
    }

    fn network_make_move(&mut self) {
        if let Some((chosen_position, output)) = self.compute() {
            // SAFETY: The compute method doesn't let the network give arbitrary positions.
            let chosen_tile = self.game.board.tile_mut(chosen_position).unwrap();

            *chosen_tile = match output.state {
                ..-0.5 => TileState::Dead,
                0.5.. => TileState::Alive,
                _ => *chosen_tile, // No move,
            };
        }
    }

    fn compute(&mut self) -> Option<(Position, KernelOutput)> {
        // Cartesian_product is smartie speech for all the unique combinations of items.
        let positions = (0..self.game.board.width)
            .cartesian_product(0..self.game.board.height)
            .map(|(x, y)| Position { x, y });

        let scored_positions = positions
            .into_iter()
            .map(|pos| (pos, self.compute_pos(pos)));

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

    fn compute_pos(&mut self, pos: Position) -> KernelOutput {
        let kernel = self.get_kernel(pos);
        let mut network_outputs = self.network_harness.compute(&kernel);

        let mut next = || {
            network_outputs
                .next()
                .expect("Not enough outputs in kernel network")
        };

        KernelOutput {
            score: next(),
            state: next(),
        }
    }

    fn get_kernel(&self, center_pos: Position) -> Kernel {
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
            .map(|maybe_position| self.game.board.tile(maybe_position?).copied())
            .collect_vec();

        Kernel { tiles }
    }

    pub fn count_alive_cells(&self) -> usize {
        self.game
            .board
            .tiles
            .iter()
            .filter(|tile| **tile == TileState::Alive)
            .count()
    }
}

pub(super) struct KernelOutput {
    /// The score by which selecting the position should be preferred.
    pub score: f32,

    // A value from -1.0 to 1.0 representing how much the network wants the tile to be alive.
    // Treated as a boolean with no side-effects.
    // NOTE: This could be changed to a tri-state with a middle-state making no move.
    pub state: f32,
}
