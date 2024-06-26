use anyhow::{bail, ensure, Context, Result};
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

    fn validate(&self) -> Result<()> {
        for (group_id, &group) in GROUPS.iter().enumerate() {
            let mut values = ValueSet::NONE;
            for loc in group {
                let cell = self.get(loc);
                if let Some(value) = cell.value() {
                    if values.contains(value) {
                        bail!("Duplicate value {value} in group {group_id}.");
                    } else {
                        values |= ValueSet::from_value(value);
                    }
                }
            }
        }
        Ok(())
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
                        ensure!(self.get(loc).is_empty(), "Location {loc} is not empty.")
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
                        if cell.is_empty() {
                            changed |= Self::restrict(cell, !ValueSet::from_value(ghost_value))
                                .with_context(|| format!("Error while restricting cell {loc} with ghost of value {ghost_value}."))?;
                        }
                    }
                }
            }
        }

        Ok(changed)
    }

    /// Generates a guess for the current state.
    /// A guess is a location and a value that is possible for that location.
    /// The location is the one with the fewest possible values left.
    ///
    /// Will return `None` if there are no empty cells left, in which case the board is solved.
    fn guess(&self) -> Option<(Location, CellValue)> {
        let location = self
            .cells
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| match cell {
                Cell::Empty(value_set) => Some((index, value_set.len())),
                Cell::Value(_) => None,
            })
            .min_by_key(|(_, len)| *len)
            .map(|(index, _)| Location::from_index(index).unwrap())?;
        let value = self.get(location).possible_values().iter().next().unwrap();
        Some((location, value))
    }
}

fn try_solve_guess(solve_state: &mut SolveState) -> Result<u32> {
    let mut steps = 0;
    while solve_state.restrict_cells().with_context(|| {
        format!(
            "Error during restrict cells step. Partial solution:\n{}",
            Board::from_solve_state(solve_state)
        )
    })? || solve_state.ghosts().with_context(|| {
        format!(
            "Error during ghosts step. Partial solution:\n{}",
            Board::from_solve_state(solve_state)
        )
    })? {
        steps += 1;
    }
    steps += 1;
    Ok(steps)
}

fn handle_error(
    stack: &mut Vec<(SolveState, Location, CellValue)>,
    error: anyhow::Error,
) -> Result<SolveState> {
    if let Some((mut prev_state, guess_loc, guess_value)) = stack.pop() {
        let guess_cell = prev_state.get_mut(guess_loc);
        SolveState::restrict(guess_cell, !ValueSet::from_value(guess_value)).with_context(
            || format!("Error updating on faulty guess at {guess_loc} with value {guess_value}."),
        )?;
        Ok(prev_state)
    } else {
        bail!(error)
    }
}

pub fn solve(board: &Board) -> Result<(Board, u32, u32)> {
    let mut stack: Vec<(SolveState, Location, CellValue)> = Vec::with_capacity(81);

    let mut cur_state = SolveState::from_board(board);
    let mut num_steps = 0;
    let mut num_guesses = 0;

    while num_steps < 1000 {
        match try_solve_guess(&mut cur_state) {
            Ok(new_steps) => num_steps += new_steps,
            Err(error) => {
                cur_state = handle_error(&mut stack, error)?;
            }
        }

        if let Some((guess_loc, guess_value)) = cur_state.guess() {
            num_guesses += 1;
            let mut guess_state = cur_state.clone();
            let guess_cell = guess_state.get_mut(guess_loc);
            *guess_cell = Cell::Value(guess_value);
            stack.push((cur_state, guess_loc, guess_value));
            cur_state = guess_state;
        } else {
            match cur_state.validate() {
                Ok(()) => return Ok((Board::from_solve_state(&cur_state), num_steps, num_guesses)),
                Err(error) => {
                    cur_state = handle_error(&mut stack, error)?;
                }
            }
        }
    }
    Ok((
        Board::from_solve_state(
            stack
                .first()
                .map(|(state, _, _)| state)
                .unwrap_or(&cur_state),
        ),
        num_steps,
        num_guesses,
    ))
}
