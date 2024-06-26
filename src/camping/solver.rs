use crate::location::{GridIter, Location};

use anyhow::{ensure, Context, Result};

use super::{map::MaybeTransposedMap, Map, Tile};
fn block_row_if_finished<M>(map: &mut M, row_index: usize, requirement: usize) -> Result<bool>
where
    M: MaybeTransposedMap,
{
    let num_tents = map
        .tiles()
        .row(row_index)
        .iter()
        .filter(|&&tile| tile == Tile::Tent)
        .count();
    if num_tents == requirement {
        let mut changed = false;
        for col_index in 0..map.width() {
            let loc = Location::new(row_index, col_index);
            changed |= map.add_blocked(loc).is_ok();
        }
        Ok(changed)
    } else {
        Ok(false)
    }
}

fn run_iter<A, M>(map: &mut M, row_index: usize, mut action: A) -> Result<()>
where
    A: FnMut(&mut M, usize, usize) -> Result<()>,
    M: MaybeTransposedMap,
{
    let width = map.width();

    let mut run_start = 0;

    for col_index in 0..width {
        let loc = Location::new(row_index, col_index);
        let cur_tile = map
            .get(loc)
            .with_context(|| format!("Location {loc} is outside of the map."))?;
        match cur_tile {
            Tile::Tree | Tile::Blocked => {
                if col_index - run_start > 0 {
                    action(map, run_start, col_index)
                        .with_context(|| format!("Error while processing run ending at {loc}."))?;
                }
                run_start = col_index + 1;
            }
            Tile::Tent => {
                assert_eq!(run_start, col_index);
                run_start = col_index + 1;
            }
            Tile::Free => {}
        }
    }
    if run_start < width {
        action(map, run_start, width).with_context(|| {
            format!("Error while processing run at end of row {row_index} starting at {run_start}.")
        })?;
    }
    Ok(())
}

fn handle_row_runs<M>(map: &mut M, row_index: usize, requirement: usize) -> Result<bool>
where
    M: MaybeTransposedMap,
{
    let mut changed = false;
    let num_possible_row_tents = map.num_possible_row_tents(row_index);
    let num_cur_row_tents = map
        .tiles()
        .row(row_index)
        .iter()
        .filter(|&&tile| tile == Tile::Tent)
        .count();
    if num_possible_row_tents == requirement - num_cur_row_tents {
        run_iter(map, row_index, |map, run_start, run_end| {
            let run_length = run_end - run_start;
            // If the run is empty, there is really no run.
            if run_length != 0 {
                // We know that at least every other cell in the run must be a tent.
                // Therefore the adjacent cells can be blocked.
                let block_locs = (run_start..run_end).flat_map(|block_col_index| {
                    [
                        (row_index > 0).then(|| Location::new(row_index - 1, block_col_index)),
                        Some(Location::new(row_index + 1, block_col_index)),
                    ]
                    .into_iter()
                    .flatten()
                });

                for block_loc in block_locs {
                    changed |= map.add_blocked(block_loc).is_ok();
                }

                // If the run is odd, we can place tents every other cell in the run,
                // and block the neighbouring cells we skipped above.
                if run_length % 2 == 1 {
                    let block_locs = [
                        (row_index > 0 && run_start > 0)
                            .then(|| Location::new(row_index - 1, run_start - 1)),
                        (row_index > 0).then(|| Location::new(row_index - 1, run_end)),
                        (run_start > 0).then(|| Location::new(row_index + 1, run_start - 1)),
                        Some(Location::new(row_index + 1, run_end)),
                    ];
                    for block_loc in block_locs.into_iter().flatten() {
                        // No need to match on the result since the below code will always set changed to true,
                        // and we don't care about the error.
                        _ = map.add_blocked(block_loc)
                    }
                    for (i, fill_col_index) in (run_start..run_end).enumerate() {
                        let fill_loc = Location::new(row_index, fill_col_index);
                        if i % 2 == 0 {
                            map.add_tent(fill_loc)
                            .with_context(|| format!("Failed to add tent. Expected position to be free. Location: {fill_loc}  Row: {row_index}"))?;
                        } else {
                            map.add_blocked(fill_loc).with_context(|| format!("Failed to add blocked. Expected position to be free. Location: {fill_loc}  Row: {row_index}"))?;
                        }
                    }
                    changed = true;
                }
            }
            Ok(())
        })?;
    } else if num_possible_row_tents == requirement - num_cur_row_tents + 1 {
        // In this case we cannot place any tents, but we can block some tiles.
        // Specifically when there are two odd-length runs with a single cell between them.
        // Since at least one of the runs must be filled,
        // one of the run-cells adjacent to the single cell will get a tent.
        // Therefore we can block the cells adjacent to the single cell.
        let mut prev_run = None;
        run_iter(map, row_index, |map, run_start, run_end| {
            assert!(run_end - run_start > 0);
            if let Some((prev_run_start, prev_run_end)) = prev_run {
                if run_start - prev_run_end == 1
                    && (prev_run_end - prev_run_start) % 2 == 1
                    && (run_end - run_start) % 2 == 1
                {
                    let block_locs = [
                        (row_index > 0).then(|| Location::new(row_index - 1, prev_run_end)),
                        Some(Location::new(row_index + 1, prev_run_end)),
                    ];
                    for block_loc in block_locs.into_iter().flatten() {
                        changed |= map.add_blocked(block_loc).is_ok()
                    }
                }
            }
            prev_run = Some((run_start, run_end));
            Ok(())
        })?;
    }
    Ok(changed)
}

fn handle_rows(map: &mut impl MaybeTransposedMap) -> Result<bool> {
    let mut changed = false;
    let row_requirements = map.row_requirements().clone();
    for (row_index, requirement) in row_requirements.into_iter().enumerate() {
        changed |= handle_row_runs(map, row_index, requirement)
            .with_context(|| format!("Error while processing runs in row {row_index}."))?;
        changed |= block_row_if_finished(map, row_index, requirement).with_context(|| {
            format!("Error while checking whether row {row_index} was finished.")
        })?;
    }
    Ok(changed)
}

pub fn fill_tents(map: &mut Map) -> Result<bool> {
    let mut changed = false;
    let old_map = map.clone();
    changed |= handle_rows(map).context("Error while filling tents in rows.")?;
    changed |=
        handle_rows(&mut map.transpose()).context("Error while filling tents in columns.")?;
    assert_eq!(changed, old_map != *map);
    Ok(changed)
}

pub fn presolve(map: &mut Map) -> Result<()> {
    let old_map = map.clone();
    let mut changed = false;
    for loc in Location::grid_iter(map.dim()) {
        if map.get(loc) == Some(Tile::Free)
            && (map
                .neighbors(loc)
                .into_iter()
                .filter_map(|x| x.map(|(_, tile)| tile))
                .any(|tile| tile == Tile::Tent)
                || !map
                    .adjacents(loc)
                    .into_iter()
                    .filter_map(|x| x.map(|(_, tile)| tile))
                    .any(|tile| tile == Tile::Tree))
            && map.get(loc).unwrap() == Tile::Free
        {
            map.add_blocked(loc).expect("Expected position to be free.");
            changed = true;
        }
    }

    map.is_valid()
        .with_context(|| format!("Invalid_map:\n{map}"))?;
    if changed {
        ensure!(*map != old_map, "`changed` is true but old_map == map.")
    }
    Ok(())
}

pub fn solve_step(map: &mut Map) -> Result<bool> {
    let old_map = map.clone();
    let changed = fill_tents(map).context("Error while filling tents.")?;

    map.is_valid()
        .with_context(|| format!("Invalid_map:\n{map}"))?;
    if changed {
        ensure!(old_map != *map, "`changed` is true map but old_map == map.")
    }
    Ok(changed)
}

struct GuessIter {
    location_iter: GridIter,
}

impl GuessIter {
    fn new(map: &Map) -> Self {
        Self {
            location_iter: Location::grid_iter(map.dim()),
        }
    }

    fn next(&mut self, map: &Map) -> Option<(Location, bool)> {
        for loc in &mut self.location_iter {
            if map.get(loc) == Some(Tile::Free) {
                return Some((loc, true));
            }
        }
        None
    }
}

fn next_try(stack: &mut Vec<(Map, GuessIter)>) -> Option<Map> {
    let mut new_map = None;
    while new_map.is_none() {
        if let Some((prev_map, mut guess_iter)) = stack.pop() {
            if let Some((loc, tile)) = guess_iter.next(&prev_map) {
                let mut map = prev_map.clone();
                if tile {
                    map.add_tent(loc).expect("Expected to add tent.");
                } else {
                    map.add_blocked(loc).expect("Expected to add blocked.");
                }
                new_map = Some(map);
                stack.push((prev_map, guess_iter));
            }
        } else {
            return None;
        }
    }
    Some(new_map.unwrap())
}

pub fn solve(map: &Map) -> Result<Option<Map>> {
    let mut map = map.clone();
    presolve(&mut map).context("Error while presolving.")?;
    let mut stack: Vec<(Map, GuessIter)> = vec![];

    let mut cur_map = map;

    loop {
        let changed = solve_step(&mut cur_map).context("Error while performing solve step.")?;
        if cur_map.is_valid().is_err() {
            cur_map = if let Some(next_map) = next_try(&mut stack) {
                next_map
            } else {
                return Ok(None);
            }
        } else if cur_map.is_complete() {
            return Ok(Some(cur_map));
        } else if !changed {
            let mut guess_iter = GuessIter::new(&cur_map);
            if let Some((loc, tile)) = guess_iter.next(&cur_map) {
                let mut map = cur_map.clone();
                if tile {
                    map.add_tent(loc).expect("Expected to add tent.");
                } else {
                    map.add_blocked(loc).expect("Expected to add blocked.");
                }
                stack.push((cur_map, guess_iter));
                cur_map = map;
            } else {
                cur_map = if let Some(next_map) = next_try(&mut stack) {
                    next_map
                } else {
                    return Ok(None);
                }
            }
        }
    }
}
