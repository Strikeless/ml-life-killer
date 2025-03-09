use libgame::board::TileState;

use crate::network::harness::InputProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kernel {
    pub tiles: Vec<Option<TileState>>,
}

impl Kernel {
    pub fn input_providers(
        kernel_diameter: usize,
    ) -> impl Iterator<Item = Box<dyn InputProvider<Self>>> {
        let tile_count = kernel_diameter.pow(2);

        (0..tile_count).map(|tile_index| {
            // Rust can't yet infer the lifetime of this closure so we need to explicitly tell that it's unbounded.
            let input_provider: impl for<'a> InputProvider<Self> =
                move |kernel| Self::input_provider(tile_index, kernel);

            // Also needs a bit help here.
            let boxed_input_provider: Box<dyn InputProvider<_>> = Box::new(input_provider);
            boxed_input_provider
        })
    }

    fn input_provider(tile_index: usize, kernel: &Self) -> f32 {
        let tile = kernel
            .tiles
            .get(tile_index)
            .expect("Not enough input tiles");

        match tile {
            Some(TileState::Alive) => 1.0,
            None => -0.0,
            Some(TileState::Dead) => -1.0,
        }
    }
}
