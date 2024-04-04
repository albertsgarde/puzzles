use std::convert::identity;

use anyhow::{bail, Result};
use itertools::Itertools;
use ndarray::{Array2, ArrayView2};

use crate::location::Location;

use super::{
    board::{BoardCell, CellValue},
    group::{Group, GROUPS},
    value_set::ValueSet,
    Board,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Empty(ValueSet),
    Value(CellValue),
}

impl Cell {
    fn value(self) -> Option<CellValue> {
        match self {
            Cell::Empty(_) => None,
            Cell::Value(value) => Some(value),
        }
    }

    fn possible_values(self) -> ValueSet {
        match self {
            Cell::Empty(value_set) => value_set,
            Cell::Value(value) => ValueSet::from_value(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SolveState {
    cells: Array2<Cell>,
}

impl SolveState {
    fn from_board(board: &Board) -> Self {
        Self {
            cells: board.grid().map(|&cell| match cell {
                BoardCell::Value(value) => Cell::Value(value),
                BoardCell::Empty => Cell::Empty(ValueSet::ALL),
            }),
        }
    }

    pub fn cells(&self) -> ArrayView2<Cell> {
        self.cells.view()
    }

    fn get(&self, location: Location) -> Cell {
        self.cells[(location.row, location.col)]
    }

    fn get_mut(&mut self, location: Location) -> &mut Cell {
        &mut self.cells[(location.row, location.col)]
    }

    fn group_cells(&self, group: Group) -> [Cell; 9] {
        group.locations.map(|location| self.get(location))
    }

    fn free_values(&self, group: Group) -> ValueSet {
        !self
            .group_cells(group)
            .into_iter()
            .flat_map(Cell::value)
            .collect::<ValueSet>()
    }

    fn restrict_cells(&mut self) -> Result<bool> {
        let mut changed = false;
        for group in GROUPS {
            let free_values = self.free_values(group);
            for loc in group {
                let cell = self.get_mut(loc);
                if let Cell::Empty(mut value_set) = *cell {
                    let start_value_set = value_set;
                    value_set &= free_values;
                    if value_set == ValueSet::NONE {
                        bail!("No possible values left for cell at {loc}.");
                    } else if let Some(single) = value_set.single() {
                        *cell = Cell::Value(single);
                        changed = true;
                    } else {
                        *cell = Cell::Empty(value_set);
                        changed |= start_value_set != value_set;
                    }
                }
            }
            let free_values = self.free_values(group);
            for value in free_values.iter() {
                if let Ok((loc, cell)) = group
                    .into_iter()
                    .map(|loc| (loc, self.get(loc)))
                    .filter(|&(_, cell)| cell.possible_values().contains(value))
                    .exactly_one()
                {
                    assert!(cell.value().is_none());
                    *self.get_mut(loc) = Cell::Value(value);
                    changed = true;
                }
            }
        }
        Ok(changed)
    }
}

pub fn solve(board: &Board) -> Board {
    let mut solve_state = SolveState::from_board(board);
    while solve_state.restrict_cells().is_ok_and(identity) {}
    Board::from_solve_state(&solve_state)
}
