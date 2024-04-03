use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub const fn transpose(self) -> Self {
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

    pub const fn grid_iter(map_dim: (usize, usize)) -> GridIter {
        GridIter::new(map_dim)
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

pub struct GridIter {
    map_dim: (usize, usize),
    cur: usize,
}

impl GridIter {
    pub const fn new(map_dim: (usize, usize)) -> Self {
        Self { map_dim, cur: 0 }
    }
}

impl Iterator for GridIter {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        let (max_row, max_col) = self.map_dim;
        if self.cur < max_row * max_col {
            let loc = Location::new(self.cur / max_col, self.cur % max_col);
            self.cur += 1;
            Some(loc)
        } else {
            None
        }
    }
}
