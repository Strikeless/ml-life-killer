use itertools::Itertools;

use super::pos::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameBoard {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileState>,
}

impl GameBoard {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![TileState::default(); width * height];
        Self::with_tiles(width, height, tiles)
    }

    pub fn new_random(width: usize, height: usize, alive_cells: usize, block_size: usize) -> Self {
        let mut board = Self::new(width, height);

        let mut available_board_positions = (0..=board.width - block_size)
            .cartesian_product(0..=board.height - block_size)
            .map(|(x, y)| Position { x, y })
            .collect_vec();

        for _ in 0..alive_cells {
            let block_root_position = {
                if available_board_positions.is_empty() {
                    panic!("Board size too small for requested alive cell count");
                }

                let chosen_position_index = rand::random_range(0..available_board_positions.len());

                // FIXME: Only the root position of blocks are removed from available cells.
                //        This whole block thing is really hackily implemented here overall, may want to recode.
                available_board_positions.swap_remove(chosen_position_index)
            };

            for block_rel_x in 0..block_size {
                for block_rel_y in 0..block_size {
                    let tile_position = block_root_position + Position {
                        x: block_rel_x,
                        y: block_rel_y,
                    };

                    // SAFETY: As long as available_board_positions only returns valid positions, we're good.
                    *board.tile_mut(tile_position).unwrap() = TileState::Alive;
                }
            }
        }

        board
    }

    pub fn with_tiles(width: usize, height: usize, tiles: Vec<TileState>) -> Self {
        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn tile<P>(&self, pos: P) -> Option<&TileState>
    where
        P: Into<Position>,
    {
        let index = self.pos_to_index(pos)?;
        self.tiles.get(index)
    }

    pub fn tile_mut<P>(&mut self, pos: P) -> Option<&mut TileState>
    where
        P: Into<Position>,
    {
        let index = self.pos_to_index(pos)?;
        self.tiles.get_mut(index)
    }

    pub fn enumerate_tiles(&self) -> impl Iterator<Item = (Position, &TileState)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| (self.index_to_pos(index), tile))
    }

    fn pos_to_index<P>(&self, pos: P) -> Option<usize>
    where
        P: Into<Position>,
    {
        let Position { x, y } = pos.into();

        if x >= self.width {
            return None;
        }

        if y >= self.height {
            return None;
        }

        Some(x + (y * self.width))
    }

    fn index_to_pos(&self, index: usize) -> Position {
        let y = index / self.width;
        let x = index % self.width;
        Position { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileState {
    Alive,

    #[default]
    Dead,
}
