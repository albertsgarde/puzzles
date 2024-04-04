use std::{
    iter::Enumerate,
    marker::PhantomData,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub},
};

use bitvec::{array::BitArray, bitarr, order::Lsb0};

use super::board::Location;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationSet {
    set: BitArray<[u8; 11]>,
}

impl LocationSet {
    pub const LAST: Self = {
        let mut data = [0; 11];
        data[10] = 0b11111110;
        Self {
            set: BitArray {
                _ord: PhantomData::<_>,
                data,
            },
        }
    };

    pub const ALL: Self = {
        let mut data = [u8::MAX; 11];
        data[10] = 0b00000001;
        Self {
            set: BitArray {
                _ord: PhantomData::<_>,
                data,
            },
        }
    };

    pub const NONE: Self = Self {
        set: bitarr![const u8, Lsb0; 0; 81],
    };

    pub fn from_location(loc: Location) -> Self {
        let mut result = Self::NONE;
        result.set.set(loc.index(), true);
        result
    }

    pub const fn row(row_index: u8) -> Self {
        let mut result = Self::NONE;
        let start_index = row_index * 9;
        let end_index = start_index + 9;
        let mut cur_index = start_index;
        while cur_index < end_index {
            let byte_index = cur_index / 8;
            let bit_index = cur_index % 8;
            result.set.data[byte_index as usize] |= 1 << bit_index;
            cur_index += 1;
        }
        result
    }

    pub const fn col(col_index: u8) -> Self {
        let mut result = Self::NONE;
        let mut cur_index = col_index;
        while cur_index < 81 {
            let byte_index = cur_index / 8;
            let bit_index = cur_index % 8;
            result.set.data[byte_index as usize] |= 1 << bit_index;
            cur_index += 9;
        }
        result
    }

    pub const fn block(grid_index: u8) -> Self {
        let mut result = Self::NONE;
        let start_row = grid_index / 3 * 3;
        let start_col = grid_index % 3 * 3;
        let end_row = start_row + 3;
        let end_col = start_col + 3;
        let mut cur_row = start_row;
        while cur_row < end_row {
            let mut cur_col = start_col;
            while cur_col < end_col {
                let index = cur_row * 9 + cur_col;
                let byte_index = index / 8;
                let bit_index = index % 8;
                result.set.data[byte_index as usize] |= 1 << bit_index;
                cur_col += 1;
            }
            cur_row += 1;
        }
        result
    }

    pub fn count(self) -> usize {
        self.set.count_ones()
    }

    pub fn is_superset(self, other: Self) -> bool {
        (self.set & other.set) == other.set
    }

    pub fn iter(self) -> LocationSetIter {
        LocationSetIter {
            iter: self.set.into_iter().enumerate(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocationSetIter {
    iter: Enumerate<bitvec::array::IntoIter<[u8; 11], Lsb0>>,
}

impl Iterator for LocationSetIter {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = None;
        while result.is_none() {
            let (index, value) = self.iter.next()?;
            if value {
                result = Some(Location::from_index(index).unwrap());
            }
        }
        result
    }
}

impl IntoIterator for LocationSet {
    type Item = Location;
    type IntoIter = LocationSetIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FromIterator<Location> for LocationSet {
    fn from_iter<I: IntoIterator<Item = Location>>(iter: I) -> Self {
        let mut result = Self::NONE;
        for loc in iter {
            result.set.set(loc.index(), true);
        }
        result
    }
}

impl Not for LocationSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            set: !self.set & !Self::LAST.set,
        }
    }
}

impl Sub<Location> for LocationSet {
    type Output = Self;

    fn sub(self, rhs: Location) -> Self::Output {
        Self {
            set: self.set & !Self::from_location(rhs).set,
        }
    }
}

impl Sub<Self> for LocationSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            set: self.set & !rhs.set,
        }
    }
}

impl BitOr for LocationSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            set: self.set | rhs.set,
        }
    }
}

impl BitOrAssign for LocationSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.set |= rhs.set;
    }
}

impl BitAnd for LocationSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            set: self.set & rhs.set,
        }
    }
}

impl BitAndAssign for LocationSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.set &= rhs.set;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn row_set() {
        for i in 0..9 {
            let loc_set = LocationSet::row(i);
            let loc_array = Location::row(i);
            assert_eq!(loc_set.count(), 9, "Row set {i} does not have 9 locations.");
            assert_eq!(
                loc_set.into_iter().count(),
                9,
                "Row set iter {i} does not have 9 locations."
            );
            for (j, (set_loc, array_loc)) in loc_set.iter().zip(loc_array.into_iter()).enumerate() {
                assert_eq!(set_loc, array_loc, "Set location {set_loc} does not match array location {array_loc} for row {i} and index {j}.");
            }
        }
    }

    #[test]
    fn col_set() {
        for i in 0..9 {
            let loc_set = LocationSet::col(i);
            let loc_array = Location::col(i);
            assert_eq!(loc_set.count(), 9, "Col set {i} does not have 9 locations.");
            assert_eq!(
                loc_set.into_iter().count(),
                9,
                "Col set iter {i} does not have 9 locations."
            );
            for (j, (set_loc, array_loc)) in loc_set.iter().zip(loc_array.into_iter()).enumerate() {
                assert_eq!(set_loc, array_loc, "Set location {set_loc} does not match array location {array_loc} for column {i} and index {j}.");
            }
        }
    }

    #[test]
    fn block_set() {
        for i in 0..9 {
            let loc_set = LocationSet::block(i);
            let loc_array = Location::block(i);
            assert_eq!(
                loc_set.count(),
                9,
                "Block set {i} does not have 9 locations."
            );
            assert_eq!(
                loc_set.into_iter().count(),
                9,
                "Block set iter {i} does not have 9 locations."
            );

            for (j, (set_loc, array_loc)) in loc_set.iter().zip(loc_array.into_iter()).enumerate() {
                assert_eq!(set_loc, array_loc, "Set location {set_loc} does not match array location {array_loc} for block {i} and index {j}.");
            }
        }
    }
}
