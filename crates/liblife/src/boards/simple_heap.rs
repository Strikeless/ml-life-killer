use crate::{LifeBoard, LifeTilePos, LifeTileState, tile_map::LifeTileMap, window::{LifeView, LifeViewMut}};

pub struct SimpleHeapLifeBoard {
    width: u32,
    height: u32,
    // NOTE: Despite being named a map, LifeTileMap is vector-backed, translating tile positions to indices.
    //       Look at SimpleSparseLifeBoard for a hashmap based board implementation.
    tile_map: LifeTileMap,
}

impl LifeBoard for SimpleHeapLifeBoard {
}

impl LifeView for SimpleHeapLifeBoard {
    type Pos = LifeTilePos;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn tile(&self, tile_pos: Self::Pos) -> LifeTileState {
        *self.tile_map.get(tile_pos)
    }
}

impl LifeViewMut for SimpleHeapLifeBoard {
    fn set_tile(&mut self, tile_pos: Self::Pos, tile_state: LifeTileState) {
        self.tile_map.set(tile_pos, tile_state);
    }
}

impl SimpleHeapLifeBoard {
    pub fn new(width: u32, height: u32) -> Self {
        let tile_map = LifeTileMap::new_filled(width, height, LifeTileState::default());

        Self {
            width,
            height,
            tile_map,
        }
    }
}
