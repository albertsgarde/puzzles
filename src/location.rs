use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn adjacents(&self, map_dim: (usize, usize)) -> [Option<Location>; 4] {
        let Self { row, col } = *self;
        let (max_row, max_col) = map_dim;
        [
            (row > 0).then(|| Location::new(row - 1, col)),
            (col < max_col - 1).then(|| Location::new(row, col + 1)),
            (row < max_row - 1).then(|| Location::new(row + 1, col)),
            (col > 0).then(|| Location::new(row, col - 1)),
        ]
    }

    pub fn neighbors(&self, map_dim: (usize, usize)) -> [Option<Location>; 8] {
        let Self { row, col } = *self;
        let (max_row, max_col) = map_dim;
        [
            (row > 0).then(|| Location::new(row - 1, col)),
            (row > 0 && col < max_col).then(|| Location::new(row - 1, col + 1)),
            (col < max_col - 1).then(|| Location::new(row, col + 1)),
            (row < max_row - 1 && col < max_col - 1).then(|| Location::new(row + 1, col + 1)),
            (row < max_row - 1).then(|| Location::new(row + 1, col)),
            (row < max_row - 1 && col > 0).then(|| Location::new(row + 1, col - 1)),
            (col > 0).then(|| Location::new(row, col - 1)),
            (row > 0 && col > 0).then(|| Location::new(row - 1, col - 1)),
        ]
    }
}
