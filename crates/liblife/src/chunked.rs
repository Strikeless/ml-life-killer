use crate::{LifeTileState, new_position_type, position::LifePositionTrait, window::{LifeView, LifeViewMut}};

new_position_type! {
    pub struct LifeChunkPos;
    pub struct LifeChunkTilePos;
}

pub struct LifeChunkView<'a, View: LifeView> {
    outer_view: &'a View,
    origin_tile_pos: View::Pos,
    width: u32,
    height: u32,
}
impl<View: LifeView> LifeView for LifeChunkView<'_, View> {
    type Pos = LifeChunkTilePos;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn tile(&self, tile_pos: Self::Pos) -> LifeTileState {
        let outer_tile_pos = self.origin_tile_pos.reduce(&tile_pos, u32::strict_add);
        self.outer_view.tile(outer_tile_pos)
    }
}

pub struct LifeChunkViewMut<'a, View: LifeViewMut> {
    outer_view: &'a mut View,
    origin_tile_pos: View::Pos,
    width: u32,
    height: u32,
}
impl<View: LifeViewMut> LifeView for LifeChunkViewMut<'_, View> {
    type Pos = LifeChunkTilePos;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn tile(&self, tile_pos: Self::Pos) -> LifeTileState {
        let outer_tile_pos = self.origin_tile_pos.reduce(&tile_pos, u32::strict_add);
        self.outer_view.tile(outer_tile_pos)
    }
}
impl<View: LifeViewMut> LifeViewMut for LifeChunkViewMut<'_, View> {
    fn set_tile(&mut self, tile_pos: Self::Pos, tile_state: LifeTileState) {
        let outer_tile_pos = self.origin_tile_pos.reduce(&tile_pos, u32::strict_add);
        self.outer_view.set_tile(outer_tile_pos, tile_state);
    }
}

pub struct LifeViewChunkAccessor<'a, View: LifeView> {
    view: &'a View,
    chunk_max_width: u32,
    chunk_max_height: u32,
}
impl<'a, View: LifeView> LifeViewChunkAccessor<'a, View> {
    pub fn new(view: &'a View, chunk_max_width: u32, chunk_max_height: u32) -> Self {
        Self {
            view,
            chunk_max_width,
            chunk_max_height,
        }
    }

    pub fn at(&mut self, chunk_pos: LifeChunkPos) -> LifeChunkView<'a, View> {
        self.at_checked(chunk_pos)
            .expect("Chunk position out of bounds")
    }

    pub fn at_checked(&mut self, chunk_pos: LifeChunkPos) -> Option<LifeChunkView<'a, View>> {
        let origin_tile_pos = self.chunk_pos_to_origin_tile_pos(chunk_pos);

        let outer_view_width = self.view.width();
        let outer_view_height = self.view.height();

        if origin_tile_pos.x() >= outer_view_width || origin_tile_pos.y() >= outer_view_height {
            return None;
        }

        // The last (rightmost and bottom) chunks on each axis may end up narrower/shorter than the wanted chunk size,
        // since the width/height of the outer view might not be exactly divisible by the wanted chunk size.
        let width = self.chunk_max_width.min(outer_view_width - origin_tile_pos.x());
        let height = self.chunk_max_height.min(outer_view_height - origin_tile_pos.y());

        let chunk_view = LifeChunkView {
            outer_view: self.view,
            width,
            height,
            origin_tile_pos,
        };
        Some(chunk_view)
    }

    pub fn chunks_wide(&self) -> u32 {
        self.view.width().div_ceil(self.chunk_max_width)
    }

    pub fn chunks_tall(&self) -> u32 {
        self.view.height().div_ceil(self.chunk_max_height)
    }

    fn chunk_pos_to_origin_tile_pos(&self, chunk_pos: LifeChunkPos) -> View::Pos {
        let origin_tile_x = chunk_pos.x() * self.chunk_max_width;
        let origin_tile_y = chunk_pos.y() * self.chunk_max_height;
        View::Pos::new(origin_tile_x, origin_tile_y)
    }
}

// At the moment there's just enough copy-pasting here for it to be a problem yet not enough to warrant a trait or macro or anything... Life sucks I guess.
// TODO: Do something about it, this is stupid
pub struct LifeViewChunkMutAccessor<'a, View: LifeViewMut> {
    view: &'a mut View,
    chunk_max_width: u32,
    chunk_max_height: u32,
}
impl<'a, View: LifeViewMut> LifeViewChunkMutAccessor<'a, View> {
    pub fn new(view: &'a mut View, chunk_max_width: u32, chunk_max_height: u32) -> Self {
        Self {
            view,
            chunk_max_width,
            chunk_max_height,
        }
    }

    pub fn take_at_mut(self, chunk_pos: LifeChunkPos) -> LifeChunkViewMut<'a, View> {
        self.take_at_mut_checked(chunk_pos)
            .expect("Chunk position out of bounds")
    }

    pub fn take_at_mut_checked(self, chunk_pos: LifeChunkPos) -> Option<LifeChunkViewMut<'a, View>> {
        let origin_tile_pos = self.chunk_pos_to_origin_tile_pos(chunk_pos);

        let outer_view_width = self.view.width();
        let outer_view_height = self.view.height();

        if origin_tile_pos.x() >= outer_view_width || origin_tile_pos.y() >= outer_view_height {
            return None;
        }

        // The last (rightmost and bottom) chunks on each axis may end up narrower/shorter than the wanted chunk size,
        // since the width/height of the outer view might not be exactly divisible by the wanted chunk size.
        let width = self.chunk_max_width.min(outer_view_width - origin_tile_pos.x());
        let height = self.chunk_max_height.min(outer_view_height - origin_tile_pos.y());

        let chunk_view = LifeChunkViewMut {
            outer_view: self.view,
            width,
            height,
            origin_tile_pos,
        };
        Some(chunk_view)
    }

    pub fn chunks_wide(&self) -> u32 {
        self.view.width().div_ceil(self.chunk_max_width)
    }

    pub fn chunks_tall(&self) -> u32 {
        // 0 1 2 3 4
        // 0 1 2 3 0
        self.view.height().div_ceil(self.chunk_max_height)
    }

    fn chunk_pos_to_origin_tile_pos(&self, chunk_pos: LifeChunkPos) -> View::Pos {
        let origin_tile_x = chunk_pos.x() * self.chunk_max_width;
        let origin_tile_y = chunk_pos.y() * self.chunk_max_height;
        View::Pos::new(origin_tile_x, origin_tile_y)
    }
}
