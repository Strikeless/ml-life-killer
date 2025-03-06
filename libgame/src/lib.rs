use board::{GameBoard, TileState};
use pos::Position;
use rule::Rule;

pub mod board;
pub mod pos;
pub mod rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub board: GameBoard,
    pub rule: Rule,
}

impl Game {
    pub fn new(board: GameBoard, rule: Rule) -> Self {
        Self { board, rule }
    }

    pub fn tick(&mut self) {
        let next_tiles = self
            .board
            .enumerate_tiles()
            .map(|(tile_pos, tile)| self.tick_tile(tile_pos, tile))
            .collect();

        self.board = GameBoard::with_tiles(self.board.width, self.board.height, next_tiles);
    }

    fn tick_tile(&self, tile_pos: Position, tile: &TileState) -> TileState {
        let alive_neighbor_count = self
            .tile_neighbors(tile_pos)
            .into_iter()
            .filter(|neighbor| **neighbor == TileState::Alive)
            .count();

        let alive = match tile {
            TileState::Alive => self.rule.survive.contains(&alive_neighbor_count),
            TileState::Dead => self.rule.birth.contains(&alive_neighbor_count),
        };

        if alive {
            TileState::Alive
        } else {
            TileState::Dead
        }
    }

    fn tile_neighbors(&self, tile_pos: Position) -> Vec<&TileState> {
        const NEIGHBOR_RELATIVE_POSITIONS: &'static [[isize; 2]] = &[
            [-1, -1],
            [-1, 0],
            [-1, 1],
            [0, -1],
            [0, 1],
            [1, -1],
            [1, 0],
            [1, 1],
        ];

        fn abs_pos(center_pos: usize, offset_pos: isize) -> Option<usize> {
            let abs_pos = center_pos as isize + offset_pos;

            if abs_pos < 0 {
                None
            } else {
                Some(abs_pos as usize)
            }
        }

        NEIGHBOR_RELATIVE_POSITIONS
            .iter()
            .filter_map(|rel_pos| {
                let pos = Position {
                    x: abs_pos(tile_pos.x, rel_pos[0])?,
                    y: abs_pos(tile_pos.y, rel_pos[1])?,
                };

                self.board.tile(pos)
            })
            .collect()
    }
}
