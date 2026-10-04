use std::collections::HashMap;

use crate::{LifeTilePos, LifeTileState, window::{LifeView, LifeViewMut}};

pub struct SimpleSparseLifeBoard {
    width: u32,
    height: u32,
    default_tile_state: LifeTileState,
    tile_map: HashMap<LifeTilePos, LifeTileState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimpleSparseLifeBoardOptions {
    pub majority_tile_state: LifeTileState,
    pub minority_preallocation_capacity: usize,
}
impl SimpleSparseLifeBoard {
    pub fn new(width: u32, height: u32) -> Self {
        Self::new_with_options(width, height, SimpleSparseLifeBoardOptions::default())
    }

    pub fn new_with_options(width: u32, height: u32, options: SimpleSparseLifeBoardOptions) -> Self {
        Self {
            width,
            height,
            default_tile_state: options.majority_tile_state,
            tile_map: HashMap::with_capacity(options.minority_preallocation_capacity),
        }
    }
}

impl LifeView for SimpleSparseLifeBoard {
    type Pos = LifeTilePos;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn tile(&self, tile_pos: Self::Pos) -> LifeTileState {
        self.tile_map.get(&tile_pos)
            .copied()
            .unwrap_or(self.default_tile_state)
    }
}

impl LifeViewMut for SimpleSparseLifeBoard {
    fn set_tile(&mut self, tile_pos: Self::Pos, tile_state: LifeTileState) {
        match tile_state == self.default_tile_state {
            true => self.tile_map.remove(&tile_pos),
            false => self.tile_map.insert(tile_pos, tile_state),
        };
    }

    fn fill(&mut self, tile_state: LifeTileState) {
        // Mass-mutations like this are the one place where there can be some benefit of a sparse hashmap backed board like this.
        // It's arguable whether this is semantically correct, since we don't update the default tile state for majority if individually setting tiles.
        self.default_tile_state = tile_state;
        self.tile_map.clear();
    }
}
