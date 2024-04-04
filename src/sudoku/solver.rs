use anyhow::{bail, Context, Result};
use itertools::Itertools;

use crate::sudoku::location_set::LocationSet;

use super::{
    board::{BoardCell, CellValue, Location},
    location_set::GROUPS,
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

    fn is_empty(self) -> bool {
        matches!(self, Cell::Empty(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolveState {
    cells: [Cell; 81],
}

impl SolveState {
    fn from_board(board: &Board) -> Self {
        Self {
            cells: board.cells().map(|cell| match cell {
                BoardCell::Value(value) => Cell::Value(value),
                BoardCell::Empty => Cell::Empty(ValueSet::ALL),
            }),
        }
    }

    pub fn cells(&self) -> &[Cell; 81] {
        &self.cells
    }

    fn get(&self, location: Location) -> Cell {
        self.cells[location.index()]
    }

    fn get_mut(&mut self, location: Location) -> &mut Cell {
        &mut self.cells[location.index()]
    }

    fn free_values(&self, locations: LocationSet) -> ValueSet {
        !locations
            .into_iter()
            .filter_map(|location| Cell::value(self.get(location)))
            .collect::<ValueSet>()
    }

    fn restrict(cell: &mut Cell, values: ValueSet) -> Result<bool> {
        match *cell {
            Cell::Empty(mut value_set) => {
                let start_value_set = value_set;
                value_set &= values;
                if value_set == ValueSet::NONE {
                    bail!("No possible values left for cell.");
                } else if let Some(single) = value_set.single() {
                    *cell = Cell::Value(single);
                    Ok(true)
                } else {
                    *cell = Cell::Empty(value_set);
                    Ok(start_value_set != value_set)
                }
            }
            Cell::Value(value) => {
                if values.contains(value) {
                    Ok(false)
                } else {
                    bail!("Cell value {value} is not possible according to value set {values}.");
                }
            }
        }
    }

    fn restrict_cells(&mut self) -> Result<bool> {
        let mut changed = false;
        let start_state = self.clone();
        for group in GROUPS {
            let free_values = self.free_values(group);
            for loc in group {
                let cell = self.get_mut(loc);
                changed |= cell.is_empty()
                    && Self::restrict(cell, free_values).with_context(|| {
                        format!("Error while restricting cell {loc} to values {free_values}.")
                    })?;
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
        if changed {
            assert_ne!(self, &start_state, "State should have changed.");
        } else {
            assert_eq!(self, &start_state, "State should not have changed.");
        }
        Ok(changed)
    }
    fn ghosts(&mut self) -> Result<bool> {
        let mut ghosts: Vec<(CellValue, LocationSet)> = vec![];

        for group in GROUPS {
            for value in ValueSet::ALL.iter() {
                let locations = group
                    .into_iter()
                    .filter(|&loc| self.get(loc).possible_values().contains(value))
                    .collect::<LocationSet>();
                if locations.count() == 2 || locations.count() == 3 {
                    for loc in locations {
                        assert!(self.get(loc).is_empty(), "Location {loc} is not empty.")
                    }
                    ghosts.push((value, locations));
                }
            }
        }

        let mut changed = false;
        for group in GROUPS {
            for &(ghost_value, locations) in ghosts.iter() {
                if group.is_superset(locations) {
                    for loc in group - locations {
                        let cell = self.get_mut(loc);
                        if let Some(cell_value) = cell.value() {
                            assert_ne!(cell_value, ghost_value)
                        } else {
                            changed |= Self::restrict(cell, !ValueSet::from_value(ghost_value))
                                .with_context(|| format!("Error while restricting cell {loc} with ghost of value {ghost_value}."))?;
                        }
                    }
                }
            }
        }

        Ok(changed)
    }
}

pub fn solve(board: &Board) -> Result<Board> {
    let mut solve_state = SolveState::from_board(board);

    while solve_state.restrict_cells().with_context(|| {
        format!(
            "Error during restrict cells step. Partial solution:\n{}",
            Board::from_solve_state(&solve_state)
        )
    })? || solve_state.ghosts().with_context(|| {
        format!(
            "Error during ghosts step. Partial solution:\n{}",
            Board::from_solve_state(&solve_state)
        )
    })? {}
    Ok(Board::from_solve_state(&solve_state))
}
