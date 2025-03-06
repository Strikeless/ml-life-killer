use std::cmp::Ordering;

use itertools::Itertools;
use kernel::{Kernel, KernelOutput};
use libgame::{Game, board::TileState, pos::Position};

use crate::network::Network;

pub mod kernel;

pub struct Player {
    pub network_1x1: Network<Kernel<{ 1 * 1 }>>,
    pub game: Game,

    pub network_consecutive_turns: usize,
    pub game_consecutive_turns: usize,
}

impl Player {
    pub fn play_step(&mut self) {
        for _ in 0..self.network_consecutive_turns {
            self.network_make_move();
        }

        for _ in 0..self.game_consecutive_turns {
            self.game.tick();
        }
    }

    fn network_make_move(&mut self) {
        if let Some((chosen_position, output)) = self.compute() {
            // SAFETY: The compute method doesn't let the network give arbitrary positions.
            let chosen_tile = self.game.board.tile_mut(chosen_position).unwrap();

            *chosen_tile = match output.state {
                ..-0.33 => TileState::Dead,
                0.33.. => TileState::Alive,
                _ => *chosen_tile, // No move,
            };
        }
    }

    fn compute(&self) -> Option<(Position, KernelOutput)> {
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

    fn compute_pos(&self, pos: Position) -> KernelOutput {
        // TODO: Multi-kernel sum or mul computations
        let prefer_1x1 = self.compute_pos_kernel(pos, &self.network_1x1);
        prefer_1x1
    }

    fn compute_pos_kernel<const TILE_COUNT: usize>(
        &self,
        pos: Position,
        kernel_network: &Network<Kernel<TILE_COUNT>>,
    ) -> KernelOutput {
        let kernel: Kernel<TILE_COUNT> = self.get_kernel(pos);
        let mut output_iter = kernel_network.compute_to_iter(&kernel);

        let mut next = || {
            output_iter
                .next()
                .expect("Not enough outputs in kernel network")
        };

        KernelOutput {
            score: next(),
            state: next(),
        }
    }

    fn get_kernel<const TILE_COUNT: usize>(&self, center_pos: Position) -> Kernel<TILE_COUNT> {
        let kernel_radius = TILE_COUNT.isqrt() as isize / 2;

        let mut tiles = Vec::new();
        for rel_x in -kernel_radius..=kernel_radius {
            for rel_y in -kernel_radius..=kernel_radius {
                let [x, y] = [center_pos.x as isize + rel_x, center_pos.y as isize + rel_y];

                let tile = if x < 0 || y < 0 {
                    None
                } else {
                    let pos = Position {
                        x: x as usize,
                        y: y as usize,
                    };
                    self.game.board.tile(pos).copied()
                };

                tiles.push(tile);
            }
        }

        // SAFETY: Should never fail, assuming the kernel radius maths are correct.
        let tiles: [_; TILE_COUNT] = tiles.try_into().unwrap();

        Kernel { tiles }
    }
}
