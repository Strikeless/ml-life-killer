pub mod boards;
pub mod engines;
mod rule;
mod position;
mod window;
mod tile_map;
mod tile_mut;
pub mod chunked;

pub use rule::LifeRules;
pub use {window::LifeView, window::LifeViewMut};
pub use tile_map::LifeTileMap;
pub use tile_mut::LifeViewTileMutWrapper;

new_position_type! { pub struct LifeTilePos; }
pub type LifeTileState = bool;
pub const TILE_NEIGHBOR_COUNT: usize = 8;

pub trait LifeBoard: LifeView<Pos = LifeTilePos> + LifeViewMut<Pos = LifeTilePos> {
}

pub trait LifeEngine<Board: LifeBoard> {
    fn tick(&self, board: &mut Board);
}
