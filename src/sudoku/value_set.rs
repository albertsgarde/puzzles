use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroU8,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub},
};

use bitvec::{array::BitArray, bitarr, order::Lsb0};

use super::board::CellValue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValueSet {
    possibilities: BitArray<[u16; 1]>,
}

impl ValueSet {
    pub const LAST: Self = Self {
        possibilities: bitarr![const u16, Lsb0; 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
    };

    pub const ALL: Self = Self {
        possibilities: bitarr![const u16, Lsb0; 1, 1, 1, 1, 1, 1, 1, 1, 1],
    };

    pub const NONE: Self = Self {
        possibilities: bitarr![const u16, Lsb0; 0; 9],
    };

    pub fn from_value(value: CellValue) -> Self {
        let mut possibilities = Self::NONE;
        let value: usize = value.into();
        possibilities.possibilities.set(value - 1, true);
        assert_eq!(possibilities & Self::LAST, Self::NONE);
        possibilities
    }

    pub fn contains(self, value: CellValue) -> bool {
        let value: usize = value.into();
        self.possibilities[value - 1]
    }

    pub fn iter(&self) -> impl Iterator<Item = CellValue> + '_ {
        assert_eq!(*self & Self::LAST, Self::NONE);
        self.possibilities.iter_ones().map(|index| {
            CellValue::new(
                NonZeroU8::new(u8::try_from(index).expect("Index cannot be larger than 256.") + 1)
                    .expect("Index + 1 must be larger than 0."),
            )
            .unwrap_or_else(|| panic!("Index must be less than 9, so index + 1 must be a valid cell value. index: {index}"))
        })
    }

    pub fn single(self) -> Option<CellValue> {
        if self.len() == 1 {
            Some(self.iter().next().unwrap())
        } else {
            None
        }
    }

    pub fn len(self) -> usize {
        self.possibilities.count_ones()
    }
}

impl Display for ValueSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for value in self.iter() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{}", value)?;
        }
        write!(f, "]")
    }
}

impl FromIterator<CellValue> for ValueSet {
    fn from_iter<I: IntoIterator<Item = CellValue>>(iter: I) -> Self {
        let mut possibilities = Self::NONE;
        for value in iter {
            let value: usize = value.into();
            possibilities.possibilities.set(value - 1, true);
        }
        possibilities
    }
}

impl Not for ValueSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            possibilities: !self.possibilities & !Self::LAST.possibilities,
        }
    }
}

impl Sub<CellValue> for ValueSet {
    type Output = Self;

    fn sub(self, rhs: CellValue) -> Self::Output {
        Self {
            possibilities: self.possibilities & !Self::from_value(rhs).possibilities,
        }
    }
}

impl Sub<Self> for ValueSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            possibilities: self.possibilities & !rhs.possibilities,
        }
    }
}

impl BitOr for ValueSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            possibilities: self.possibilities | rhs.possibilities,
        }
    }
}

impl BitOrAssign for ValueSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.possibilities |= rhs.possibilities;
    }
}

impl BitAnd for ValueSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            possibilities: self.possibilities & rhs.possibilities,
        }
    }
}

impl BitAndAssign for ValueSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.possibilities &= rhs.possibilities;
    }
}
