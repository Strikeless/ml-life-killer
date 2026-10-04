use crate::{LifeTileState, chunked::{LifeViewChunkAccessor, LifeViewChunkMutAccessor}, position::LifePositionTrait, tile_map::LifeTileMap, tile_mut::LifeViewTileMutWrapper};

pub trait LifeView {
    type Pos: LifePositionTrait + Copy;

    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn tile(&self, tile_pos: Self::Pos) -> LifeTileState;
    fn tile_checked(&self, tile_pos: Self::Pos) -> Option<LifeTileState> {
        if tile_pos.x() >= self.width() || tile_pos.y() >= self.height() {
            return None;
        }

        let tile_state = self.tile(tile_pos);
        Some(tile_state)
    }

    fn iter_tile_positions(&self) -> impl ExactSizeIterator<Item = Self::Pos> + 'static {
        let window_width = self.width();
        let window_height = self.height();

        (0..window_width * window_height)
            .map(move |tile_index| {
                let tile_x = tile_index % window_width;
                let tile_y = tile_index / window_width;
                Self::Pos::new(tile_x, tile_y)
            })
    }

    fn iter_tiles(&self) -> impl ExactSizeIterator<Item = LifeTileState> {
        self.iter_tile_positions()
            .map(|tile_pos| self.tile(tile_pos))
    }

    fn enumerate_tiles(&self) -> impl ExactSizeIterator<Item = (Self::Pos, LifeTileState)> {
        self.iter_tile_positions()
            .map(|tile_pos| (tile_pos, self.tile(tile_pos)))
    }

    fn to_tile_map(&self) -> LifeTileMap {
        let window_width = self.width() as usize;
        // iter_tiles and LifeTileMap conveniently use the same y-major tile order, so we can do the conversion like this.
        let map_tiles = self.iter_tiles().collect();
        LifeTileMap::new_from_values(window_width, map_tiles)
    }

    fn chunked(&self, chunk_max_width: u32, chunk_max_height: u32) -> LifeViewChunkAccessor<'_, Self> where Self: Sized {
        LifeViewChunkAccessor::new(self, chunk_max_width, chunk_max_height)
    }
}

pub trait LifeViewMut: LifeView {
    fn set_tile(&mut self, tile_pos: Self::Pos, tile_state: LifeTileState);
    fn set_tile_checked(&mut self, tile_pos: Self::Pos, tile_state: LifeTileState) -> bool {
        if tile_pos.x() >= self.width() || tile_pos.y() >= self.height() {
            return false;
        }

        self.set_tile(tile_pos, tile_state);
        true
    }

    fn tile_mut(&mut self, tile_pos: Self::Pos) -> LifeViewTileMutWrapper<'_, Self> where Self: Sized {
        let initial_tile_state = self.tile(tile_pos);
        LifeViewTileMutWrapper::new(self, tile_pos, initial_tile_state)
    }
    fn tile_mut_checked(&mut self, tile_pos: Self::Pos) -> Option<LifeViewTileMutWrapper<'_, Self>> where Self: Sized {
        if tile_pos.x() >= self.width() || tile_pos.y() >= self.height() {
            return None;
        }

        let tile_mut_wrapper = self.tile_mut(tile_pos);
        Some(tile_mut_wrapper)
    }

    fn fill(&mut self, tile_state: LifeTileState) {
        let window_width = self.width();
        let window_height = self.height();

        for tile_y in 0..window_height {
            for tile_x in 0..window_width {
                let tile_pos = Self::Pos::new(tile_x, tile_y);
                self.set_tile(tile_pos, tile_state);
            }
        }
    }

    fn chunked_mut(&mut self, chunk_max_width: u32, chunk_max_height: u32) -> LifeViewChunkMutAccessor<'_, Self> where Self: Sized {
        LifeViewChunkMutAccessor::new(self, chunk_max_width, chunk_max_height)
    }
}
