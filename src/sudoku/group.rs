use array_concat::concat_arrays;

use crate::location::Location;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Group {
    pub locations: [Location; 9],
}

impl Group {
    const fn row(row_index: usize) -> Self {
        Self {
            locations: [
                Location::new(row_index, 0),
                Location::new(row_index, 1),
                Location::new(row_index, 2),
                Location::new(row_index, 3),
                Location::new(row_index, 4),
                Location::new(row_index, 5),
                Location::new(row_index, 6),
                Location::new(row_index, 7),
                Location::new(row_index, 8),
            ],
        }
    }

    const fn col(col_index: usize) -> Self {
        Self {
            locations: [
                Location::new(0, col_index),
                Location::new(1, col_index),
                Location::new(2, col_index),
                Location::new(3, col_index),
                Location::new(4, col_index),
                Location::new(5, col_index),
                Location::new(6, col_index),
                Location::new(7, col_index),
                Location::new(8, col_index),
            ],
        }
    }

    const fn grid(grid_index: usize) -> Self {
        let start_row = (grid_index / 3) * 3;
        let start_col = (grid_index % 3) * 3;
        Self {
            locations: [
                Location::new(start_row, start_col),
                Location::new(start_row, start_col + 1),
                Location::new(start_row, start_col + 2),
                Location::new(start_row + 1, start_col),
                Location::new(start_row + 1, start_col + 1),
                Location::new(start_row + 1, start_col + 2),
                Location::new(start_row + 2, start_col),
                Location::new(start_row + 2, start_col + 1),
                Location::new(start_row + 2, start_col + 2),
            ],
        }
    }
}

impl IntoIterator for Group {
    type Item = Location;
    type IntoIter = std::array::IntoIter<Location, 9>;

    fn into_iter(self) -> Self::IntoIter {
        self.locations.into_iter()
    }
}

pub const ROWS: [Group; 9] = [
    Group::row(0),
    Group::row(1),
    Group::row(2),
    Group::row(3),
    Group::row(4),
    Group::row(5),
    Group::row(6),
    Group::row(7),
    Group::row(8),
];

pub const COLS: [Group; 9] = [
    Group::col(0),
    Group::col(1),
    Group::col(2),
    Group::col(3),
    Group::col(4),
    Group::col(5),
    Group::col(6),
    Group::col(7),
    Group::col(8),
];

pub const BLOCKS: [Group; 9] = [
    Group::grid(0),
    Group::grid(1),
    Group::grid(2),
    Group::grid(3),
    Group::grid(4),
    Group::grid(5),
    Group::grid(6),
    Group::grid(7),
    Group::grid(8),
];

pub const GROUPS: [Group; 27] = concat_arrays!(ROWS, COLS, BLOCKS);
