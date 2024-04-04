use array_concat::concat_arrays;

use super::{
    board::Location,
    location_set::{LocationSet, LocationSetIter},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Group {
    locations: LocationSet,
}

impl Group {
    const fn row(row_index: u8) -> Self {
        Self {
            locations: LocationSet::row(row_index),
        }
    }

    const fn col(col_index: u8) -> Self {
        Self {
            locations: LocationSet::col(col_index),
        }
    }

    const fn block(grid_index: u8) -> Self {
        Self {
            locations: LocationSet::block(grid_index),
        }
    }
}

impl IntoIterator for Group {
    type Item = Location;
    type IntoIter = LocationSetIter;

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
    Group::block(0),
    Group::block(1),
    Group::block(2),
    Group::block(3),
    Group::block(4),
    Group::block(5),
    Group::block(6),
    Group::block(7),
    Group::block(8),
];

pub const GROUPS: [Group; 27] = concat_arrays!(ROWS, COLS, BLOCKS);
