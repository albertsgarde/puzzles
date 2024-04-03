use anyhow::{bail, Context, Result};
use std::{fmt::{Display, Formatter, Write}, num::NonZeroU8};

use ndarray::Array2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellValue {
    Empty,
    Value(NonZeroU8),
}

#[derive(Clone, Debug)]
pub struct Board {
    cells: Array2<CellValue>,
}

impl Board {
    pub fn from_line(line: &str, empty_char: char) -> Result<Self> {
        if line.len() != 81 {
            bail!("Line must be exactly 81 characters long, but is {}. Line: '{line}'", line.len());
        }
        let cells = line
            .chars()
            .enumerate()
            .map(|(index, c)| {
                Ok(match c {
                    c if c == empty_char => CellValue::Empty,
                    c => {
                        if let Some(digit) = c.to_digit(10) {
                            CellValue::Value(NonZeroU8::new(digit.try_into().unwrap()).with_context(|| 
                                format!("Invalid digit '{digit}' at index {index} in line '{line}'. '0' is not a valid character")
                            )?)
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

        let cells: Vec<CellValue> = grid.lines().enumerate().flat_map(|(row_index, line)|{
            if line.len() != 9 {
                bail!("Line must be exactly 9 characters long, but is {}. Line: '{line}'", line.len());
            }
            Ok(line.chars().enumerate().map(move |(col_index, c)| {
                Ok(match c {
                    c if c == empty_char => CellValue::Empty,
                    c => {
                        if let Some(digit) = c.to_digit(10) {
                            CellValue::Value(NonZeroU8::new(digit.try_into().unwrap()).with_context(|| 
                                format!("Invalid digit '{digit}' at row {row_index}, column {col_index} in grid '{grid}'. '0' is not a valid character")
                            )?)
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
            match cell {
                CellValue::Empty => write!(f, "{}", empty_char)?,
                CellValue::Value(value) => write!(f, "{}", value.get())?,
            }
        }
        Ok(())
    }

    pub fn format_compact_grid(&self, f: &mut Formatter<'_>, empty_char: char) -> std::fmt::Result {
        for row in self.cells.rows() {
            for &cell in row {
                match cell {
                    CellValue::Empty => write!(f, "{}", empty_char)?,
                    CellValue::Value(value) => write!(f, "{}", value.get())?,
                }
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
                match cell {
                    CellValue::Empty => write!(f, "{} ", empty_char)?,
                    CellValue::Value(value) => write!(f, "{} ", value.get())?,
                }
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
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format_pretty_grid(f, ' ')
    }
}
