use std::marker::PhantomData;

use crate::{LifeTilePos, LifeTileState, position::LifePositionTrait};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifeTileMap<K: LifePositionTrait = LifeTilePos, V = LifeTileState> {
    width: usize,
    values: Vec<V>,
    _key_phantom: PhantomData<K>,
}

impl<K: LifePositionTrait, V> LifeTileMap<K, V> {
    pub fn new_filled(width: u32, height: u32, fill_value: V) -> Self where V: Clone {
        let tile_values = vec![
            fill_value;
            width as usize * height as usize
        ];

        Self::new_from_values(width as usize, tile_values)
    }

    pub fn new_from_values(width: usize, tile_values: Vec<V>) -> Self {
        debug_assert!(
            tile_values.len() % width == 0,
            "Value vector length isn't even with specified width",
        );
        debug_assert!(
            width <= u32::MAX as usize,
            "Specified width exceeds bounds of dimensional type u32",
        );
        debug_assert!(
            tile_values.len() / width <= u32::MAX as usize,
            "Value vector line count with specified line width exceeds bounds of dimensional type u32",
        );

        Self {
            width,
            values: tile_values,
            _key_phantom: PhantomData,
        }
    }

    pub fn get(&self, pos: K) -> &V {
        let tile_index = self.pos_to_tile_index(pos);
        &self.values[tile_index]
    }

    pub fn set(&mut self, pos: K, tile_value: V) {
        let tile_index = self.pos_to_tile_index(pos);
        self.values[tile_index] = tile_value;
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        (self.values.len() / self.width) as u32
    }

    fn pos_to_tile_index(&self, pos: K) -> usize {
        let x = pos.x() as usize;
        let y = pos.y() as usize;
        y * self.width + x
    }
}
