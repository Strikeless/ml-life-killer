use itertools::Itertools;

use crate::{LifeBoard, LifeEngine, LifeRules, LifeTilePos, LifeTileState, tile_map::LifeTileMap, position::LifePositionTrait};

#[derive(Debug, Clone)]
pub struct SimpleLifeEngine {
    rules: LifeRules,
    options: SimpleLifeEngineOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimpleLifeEngineOptions {
    pub out_of_bounds_neighbor_state: LifeTileState,
}

impl<Board> LifeEngine<Board> for SimpleLifeEngine
where
    Board: LifeBoard,
{
    fn tick(&self, board: &mut Board) {
        let board_width = board.width();
        let board_height = board.height();
        let initial_tile_state_map = board.to_tile_map();

        for tile_pos in board.iter_tile_positions() {
            let tile_state = board.tile(tile_pos);

            let tile_neighbor_states = self.get_tile_neighbor_states(
                board_width,
                board_height,
                &initial_tile_state_map,
                tile_pos
            );

            let ticked_tile_state = self.rules.compute_next_tile_state(tile_state, &tile_neighbor_states);
            board.set_tile(tile_pos, ticked_tile_state);
        }
    }
}

impl SimpleLifeEngine {
    pub fn new(rules: LifeRules) -> Self {
        Self::new_with_options(rules, SimpleLifeEngineOptions::default())
    }

    pub fn new_with_options(rules: LifeRules, options: SimpleLifeEngineOptions) -> Self {
        Self { rules, options }
    }

    fn get_tile_neighbor_states(
        &self,
        board_width: u32,
        board_height: u32,
        initial_tile_state_map: &LifeTileMap,
        center_tile_pos: LifeTilePos,
    ) -> [LifeTileState; crate::TILE_NEIGHBOR_COUNT] {
        #[rustfmt::skip]
        const NEIGHBOR_TILE_POSITION_OFFSETS: [[i32; 2]; crate::TILE_NEIGHBOR_COUNT] = [
            [-1, -1],   [0, -1],    [1, -1],
            [-1,  0],               [1,  0],
            [-1,  1],   [0,  1],    [1,  1],
        ];

        let maybe_neighbor_tile_positions = NEIGHBOR_TILE_POSITION_OFFSETS
            .into_iter()
            .map(|[tile_offset_x, tile_offset_y]| {
                let tile_x = center_tile_pos.x().checked_add_signed(tile_offset_x)?;
                let tile_y = center_tile_pos.y().checked_add_signed(tile_offset_y)?;

                if tile_x >= board_width {
                    return None;
                }
                if tile_y >= board_height {
                    return None;
                }

                Some(LifeTilePos::new(tile_x, tile_y))
            });

        maybe_neighbor_tile_positions
            .map(|maybe_neighbor_tile_pos| match maybe_neighbor_tile_pos {
                Some(neighbor_tile_pos) => *initial_tile_state_map.get(neighbor_tile_pos),
                None => self.options.out_of_bounds_neighbor_state,
            })
            .collect_array()
            .unwrap()
    }
}
