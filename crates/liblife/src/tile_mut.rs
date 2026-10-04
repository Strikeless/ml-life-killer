use std::ops::{Deref, DerefMut};

use crate::{LifeTileState, window::LifeViewMut};

#[must_use]
pub struct LifeViewTileMutWrapper<'a, View: LifeViewMut> {
    view: &'a mut View,
    tile_pos: View::Pos,
    tile_value: LifeTileState,
}

impl<'a, View: LifeViewMut> LifeViewTileMutWrapper<'a, View> {
    pub fn new(view: &'a mut View, tile_pos: View::Pos, initial_tile_value: LifeTileState) -> Self {
        Self {
            view,
            tile_pos,
            tile_value: initial_tile_value,
        }
    }
}

impl<'a, View: LifeViewMut> Deref for LifeViewTileMutWrapper<'a, View> {
    type Target = LifeTileState;

    fn deref(&self) -> &Self::Target {
        &self.tile_value
    }
}

impl<'a, View: LifeViewMut> DerefMut for LifeViewTileMutWrapper<'a, View> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tile_value
    }
}

impl<'a, View: LifeViewMut> Drop for LifeViewTileMutWrapper<'a, View> {
    fn drop(&mut self) {
        self.view.set_tile(self.tile_pos, self.tile_value);
    }
}
