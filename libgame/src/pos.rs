#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl From<[usize; 2]> for Position {
    fn from(value: [usize; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<Position> for [usize; 2] {
    fn from(value: Position) -> Self {
        [value.x, value.y]
    }
}
