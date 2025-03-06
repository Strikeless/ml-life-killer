use std::ops::{Add, Mul};

use libgame::board::TileState;

use crate::network::layer::input::InputLayer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kernel<const TILE_COUNT: usize> {
    pub tiles: [Option<TileState>; TILE_COUNT],
}

// HACK: Uhh stupid stuff figure it out and please fix.
impl<const TILE_COUNT: usize> Default for Kernel<TILE_COUNT> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<const TILE_COUNT: usize> Kernel<TILE_COUNT> {
    pub fn create_input_layer() -> InputLayer<Self> {
        let mut input_layer = InputLayer::new();

        for _tile_index in 0..TILE_COUNT {
            input_layer.add_input(|provider_index, kernel: &Self| {
                // HACK: Should use _tile_index but Rust don't let that happen with closures and fns.
                //       This will break if you fuck around with the providers without knowing.
                let tile = kernel.tiles[provider_index];

                match tile {
                    Some(TileState::Alive) => 1.0,
                    None => -0.0,
                    Some(TileState::Dead) => -1.0,
                }
            });
        }

        input_layer
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

impl Add for KernelOutput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            score: self.score + rhs.score,
            state: self.state + rhs.state,
        }
    }
}

impl Mul for KernelOutput {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            score: self.score * rhs.score,
            state: self.state * rhs.state,
        }
    }
}
