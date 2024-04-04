use anyhow::{bail, Context, Result};
use std::{ fmt::{Display, Formatter, Write}, num::NonZeroU8};

use ndarray::{Array2, ArrayView2};

use super::solver::{Cell, SolveState};


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

#[derive(Clone, Debug)]
pub struct Board {
    cells: Array2<BoardCell>,
}

impl Board {
    pub fn from_solve_state(solve_state: &SolveState) -> Self {
        Self {
            cells: solve_state.cells().map(|&cell| match cell {
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
        let cells = Array2::from_shape_vec((9, 9), cells).unwrap();
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
        let cells = Array2::from_shape_vec((9, 9), cells).unwrap();
        Ok(Self { cells })
    }

    pub fn format_line(&self, f: &mut Formatter<'_>, empty_char: char) -> std::fmt::Result {
        for &cell in self.cells.iter() {
            write!(f, "{}", cell.to_char(empty_char))?;
        }
        Ok(())
    }

    pub fn format_compact_grid(&self, f: &mut Formatter<'_>, empty_char: char) -> std::fmt::Result {
        for row in self.cells.rows() {
            for &cell in row {
                write!(f, "{}", cell.to_char(empty_char))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }

    pub fn format_pretty_grid(&self, f: &mut impl Write, empty_char: char) -> std::fmt::Result {
        for (row_index, row) in self.cells.rows().into_iter().enumerate() {
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

    pub fn to_pretty_string<F>(&self, format: F, empty_char: char) -> String where F: FnOnce(&Self, &mut String, char)-> std::fmt::Result {
        let mut s = String::new();
        format(self, &mut s, empty_char).unwrap();
        s
    }

    pub fn grid(&self) -> ArrayView2<BoardCell> {
        self.cells.view()
    
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
