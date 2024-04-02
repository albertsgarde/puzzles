use std::fmt::{self, Display, Formatter};

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

    pub fn transpose(self) -> Self {
        Self {
            row: self.col,
            col: self.row,
        }
    }

    pub fn adjacents(self, map_dim: (usize, usize)) -> [Option<Location>; 4] {
        let Self { row, col } = self;
        let (max_row, max_col) = map_dim;
        [
            (row > 0).then(|| Location::new(row - 1, col)),
            (col < max_col - 1).then(|| Location::new(row, col + 1)),
            (row < max_row - 1).then(|| Location::new(row + 1, col)),
            (col > 0).then(|| Location::new(row, col - 1)),
        ]
    }

    pub fn neighbors(self, map_dim: (usize, usize)) -> [Option<Location>; 8] {
        let Self { row, col } = self;
        let (max_row, max_col) = map_dim;
        [
            (row > 0).then(|| Location::new(row - 1, col)),
            (row > 0 && col < max_col - 1).then(|| Location::new(row - 1, col + 1)),
            (col < max_col - 1).then(|| Location::new(row, col + 1)),
            (row < max_row - 1 && col < max_col - 1).then(|| Location::new(row + 1, col + 1)),
            (row < max_row - 1).then(|| Location::new(row + 1, col)),
            (row < max_row - 1 && col > 0).then(|| Location::new(row + 1, col - 1)),
            (col > 0).then(|| Location::new(row, col - 1)),
            (row > 0 && col > 0).then(|| Location::new(row - 1, col - 1)),
        ]
    }

    pub fn grid_iter(map_dim: (usize, usize)) -> impl Iterator<Item = Location> {
        let (max_row, max_col) = map_dim;
        (0..max_row).flat_map(move |row| (0..max_col).map(move |col| Location::new(row, col)))
    }
}

impl From<(usize, usize)> for Location {
    fn from((row, col): (usize, usize)) -> Self {
        Self { row, col }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}
