use anyhow::{bail, Context, Result};
use thiserror::Error;
use std::{ fmt::{Display, Formatter, Write}, num::NonZeroU8};


use super::{group, solver::{Cell, SolveState}, value_set::ValueSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    index: u8,
}

impl Location {
    pub const fn new(row: u8, col: u8) -> Option<Self> {
        if row < 9 && col < 9 {
            Some(Self { index: row * 9 + col })
        } else {
            None
        }
    }

    pub const fn row(row_index: u8) -> [Self; 9] {
        [
            Self {index: row_index * 9},
            Self {index: row_index * 9 + 1},
            Self {index: row_index * 9 + 2},
            Self {index: row_index * 9 + 3},
            Self {index: row_index * 9 + 4},
            Self {index: row_index * 9 + 5},
            Self {index: row_index * 9 + 6},
            Self {index: row_index * 9 + 7},
            Self {index: row_index * 9 + 8},
        ]
    }

    pub const fn col(col_index: u8) -> [Self; 9] {
        [
            Self {index: col_index},
            Self {index: col_index + 9},
            Self {index: col_index + 18},
            Self {index: col_index + 27},
            Self {index: col_index + 36},
            Self {index: col_index + 45},
            Self {index: col_index + 54},
            Self {index: col_index + 63},
            Self {index: col_index + 72},
        ]
    }

    pub const fn block(block_index: u8) -> [Self; 9] {
        let start_row = (block_index / 3) * 3;
        let start_col = (block_index % 3) * 3;
        [
            Self {index: start_row * 9 + start_col},
            Self {index: start_row * 9 + start_col + 1},
            Self {index: start_row * 9 + start_col + 2},
            Self {index: (start_row + 1) * 9 + start_col},
            Self {index: (start_row + 1) * 9 + start_col + 1},
            Self {index: (start_row + 1) * 9 + start_col + 2},
            Self {index: (start_row + 2) * 9 + start_col},
            Self {index: (start_row + 2) * 9 + start_col + 1},
            Self {index: (start_row + 2) * 9 + start_col + 2},
        ]
    }

    pub const fn to_row_col(self) -> (u8, u8) {
        let index = self.index;
        (index / 9, index % 9)
    }

    pub const fn index(self) -> usize {
        self.index as usize
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (row, col) = self.to_row_col();
        write!(f, "({}, {})", row, col)
    }

}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellValue {
    value: NonZeroU8,
}

impl CellValue {
    pub fn new(value: NonZeroU8) -> Option<Self> {
        (value <= NonZeroU8::new(9).unwrap()).then_some(Self {value})
    }

    pub fn to_char(self) -> char {
        char::from_digit(self.value.get().into(), 10).unwrap()
    }
}

impl From<CellValue> for NonZeroU8 {
    fn from(value: CellValue) -> Self {
        value.value
    }
}

impl From<CellValue> for u8 {
    fn from(value: CellValue) -> Self {
        value.value.get()
    }
}

impl From<CellValue> for usize {
    fn from(value: CellValue) -> Self {
        value.value.get() as usize
    }
}

impl Display for CellValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoardCell {
    Empty,
    Value(CellValue),
}

impl BoardCell {
    pub fn to_char(self, empty_char: char) -> char {
        match self {
            BoardCell::Empty => empty_char,
            BoardCell::Value(value) => value.to_char(),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
pub enum InvalidBoardError {
    #[error("Row {row_index} has duplicate value {value}")]
    DuplicateRowValue { row_index: usize, value: CellValue },
    #[error("Column {col_index} has duplicate value {value}")]
    DuplicateColumnValue { col_index: usize, value: CellValue },
    #[error("Block {block_index} has duplicate value {value}")]
    DuplicateBlockValue { block_index: usize, value: CellValue },
}

#[derive(Clone, Debug)]
pub struct Board {
    cells: [BoardCell; 81],
}

impl Board {
    pub fn from_solve_state(solve_state: &SolveState) -> Self {
        Self {
            cells: solve_state.cells().map(|cell| match cell {
                Cell::Value(value) => BoardCell::Value(value),
                Cell::Empty(_) => BoardCell::Empty,
            }),
        }
    }

    pub fn from_line(line: &str, empty_char: char) -> Result<Self> {
        if line.len() != 81 {
            bail!("Line must be exactly 81 characters long, but is {}. Line: '{line}'", line.len());
        }
        let cells = line
            .chars()
            .enumerate()
            .map(|(index, c)| {
                Ok(match c {
                    c if c == empty_char => BoardCell::Empty,
                    c => {
                        if let Some(digit) = c.to_digit(10) {
                            BoardCell::Value(CellValue::new(NonZeroU8::new(digit.try_into().unwrap()).with_context(|| 
                                format!("Invalid digit '{digit}' at index {index} in line '{line}'. '0' is not a valid character"))?
                            ).unwrap())
                        } else {
                            bail!("Invalid character '{c}' at index {index} in line '{line}'.")
                        }
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let cells: [BoardCell; 81] = cells.try_into().unwrap();
        Ok(Self { cells })
    }

    pub fn from_grid(grid: &str, empty_char: char) -> Result<Self> {
        if grid.len() != 90 {
            bail!("Grid must be exactly 90 characters long (81 for grid, 9 for newlines), but is {}. Grid: '{grid}'", grid.len());
        }

        let cells: Vec<BoardCell> = grid.lines().enumerate().flat_map(|(row_index, line)|{
            if line.len() != 9 {
                bail!("Line must be exactly 9 characters long, but is {}. Line: '{line}'", line.len());
            }
            Ok(line.chars().enumerate().map(move |(col_index, c)| {
                Ok(match c {
                    c if c == empty_char => BoardCell::Empty,
                    c => {
                        if let Some(digit) = c.to_digit(10) {
                            BoardCell::Value(CellValue::new(NonZeroU8::new(digit.try_into().unwrap()).with_context(|| 
                                format!("Invalid digit '{digit}' at row {row_index}, column {col_index} in grid '{grid}'. '0' is not a valid character")
                            )?).unwrap())
                        } else {
                            bail!("Invalid character '{c}' at row {row_index}, column {col_index} in grid '{grid}'.")
                        }
                    }
                })
            }))
        }).flatten().collect::<Result<_>>()?;
        let cells: [BoardCell; 81] = cells.try_into().unwrap();
        Ok(Self { cells })
    }

    pub fn format_line(&self, f: &mut impl Write, empty_char: char) -> std::fmt::Result {
        for &cell in self.cells.iter() {
            write!(f, "{}", cell.to_char(empty_char))?;
        }
        Ok(())
    }

    pub fn format_compact_grid(&self, f: &mut impl Write, empty_char: char) -> std::fmt::Result {
        for row in self.cells.chunks_exact(9) {
            for &cell in row {
                write!(f, "{}", cell.to_char(empty_char))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }

    pub fn format_pretty_grid(&self, f: &mut impl Write, empty_char: char) -> std::fmt::Result {
        for (row_index, row) in self.cells.chunks_exact(9).enumerate() {
            if row_index % 3 == 0 {
                writeln!(f, "+-------+-------+-------+")?;
            }
            for (col_index, &cell) in row.iter().enumerate() {
                if col_index % 3 == 0 {
                    write!(f, "| ")?;
                }
                write!(f, "{} ", cell.to_char(empty_char))?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "+-------+-------+-------+")?;
        Ok(())
    }

    pub fn to_pretty_string<F>(&self, format: F, empty_char: char) -> Result<String, std::fmt::Error> where F: FnOnce(&Self, &mut String, char) -> std::fmt::Result {
        let mut s = String::new();
        format(self, &mut s, empty_char)?;
        Ok(s)
    }

    pub fn cells(&self) -> &[BoardCell; 81] {
        &self.cells
    }

    pub fn get(&self, loc: Location) -> BoardCell {
        self.cells[loc.index()]
    }

    pub fn validate(&self) -> Result<&Self, InvalidBoardError> {
        // Validate rows
        for (row_index, row) in group::ROWS.into_iter().enumerate() {
            let mut values = ValueSet::NONE;
            for cell in row.into_iter().map(|location| self.get(location)) {
                if let BoardCell::Value(value) = cell {
                    if values.contains(value) {
                        return Err(InvalidBoardError::DuplicateRowValue { row_index, value });
                    }
                    values |= ValueSet::from_value(value);
                }
            }
        }

        // Validate columns
        for (col_index, col) in group::COLS.into_iter().enumerate() {
            let mut values = ValueSet::NONE;
            for cell in col.into_iter().map(|location| self.get(location)) {
                if let BoardCell::Value(value) = cell {
                    if values.contains(value) {
                        return Err(InvalidBoardError::DuplicateColumnValue { col_index, value });
                    }
                    values |= ValueSet::from_value(value);
                }
            }
        }

        // Validate blocks
        for (block_index, block) in group::BLOCKS.into_iter().enumerate() {
            let mut values = ValueSet::NONE;
            for cell in block.into_iter().map(|location| self.get(location)) {
                if let BoardCell::Value(value) = cell {
                    if values.contains(value) {
                        return Err(InvalidBoardError::DuplicateBlockValue { block_index, value });
                    }
                    values |= ValueSet::from_value(value);
                }
            }
        }

        Ok(self)
    }

    pub fn finished(&self) -> bool {
        self.cells.iter().all(|&cell| cell != BoardCell::Empty)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format_pretty_grid(f, ' ')
    }
}
