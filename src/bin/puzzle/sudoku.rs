use std::path::Path;

use anyhow::{Context, Result};
use puzzles::sudoku::{self, Board};

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("data/sudoku")
}

fn read_boards_from_lines<S: AsRef<str>>(
    lines: impl Iterator<Item = S>,
    empty_char: char,
) -> Result<Vec<Board>> {
    lines
        .map(|line| Board::from_line(line.as_ref(), empty_char))
        .collect::<Result<Vec<_>>>()
}

fn load_grid_file(file: impl AsRef<Path>) -> Result<Vec<Board>> {
    let file = file.as_ref();
    let data_str = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read grid file '{file:?}'."))?;
    read_boards_from_lines(data_str.lines(), '.')
}

fn load_qqwing(postfix: impl AsRef<str>) -> Result<Vec<Board>> {
    let grid_dir = data_dir().join("grids");
    let file_path = grid_dir
        .join(format!("qqwing_{}", postfix.as_ref()))
        .with_extension("txt");
    load_grid_file(file_path)
}

pub fn solve_grid_group(grids: impl AsRef<[Board]>, descriptor: impl AsRef<str>) -> Result<()> {
    let grids = grids.as_ref();
    let descriptor = descriptor.as_ref();
    let num_grids = grids.len();
    let mut num_solved = 0;
    for grid in grids {
        let solution =
            sudoku::solve(grid).with_context(|| format!("Error while solving board:\n{grid}"))?;
        if solution.validate().with_context(||
                format!("Error validating partial solution.\nPartial solution:\n{solution}Original board:\n{grid}")
            )?.finished() {
            num_solved += 1;
        }
    }

    let percentage = num_solved as f64 / num_grids as f64 * 100.0;
    println!("Solved {num_solved}/{num_grids} ({percentage:.0}%) {descriptor} grids.",);
    Ok(())
}
#[derive(Clone, Debug, clap::Args)]
pub struct Sudoku {}

impl Sudoku {
    pub fn run(self) -> Result<()> {
        let grid_dir = data_dir().join("grids");
        let qqwing_simple_grids =
            load_qqwing("simple").context("Error loading simple qqwing grids")?;
        let qqwing_easy_grids = load_qqwing("easy").context("Error loading easy qqwing grids")?;
        let qqwing_intermediate_grids =
            load_qqwing("intermediate").context("Error loading intermediate qqwing grids")?;
        let qqwing_expert_grids =
            load_qqwing("expert").context("Error loading expert qqwing grids")?;
        let easy50_grids =
            load_grid_file(grid_dir.join("easy50.txt")).context("Error loading easy50 grids")?;
        let top95_grids =
            load_grid_file(grid_dir.join("top95.txt")).context("Error loading top95 grids")?;
        let hardest_grids =
            load_grid_file(grid_dir.join("hardest.txt")).context("Error loading hardest grids")?;

        solve_grid_group(qqwing_simple_grids, "simple")?;
        solve_grid_group(qqwing_easy_grids, "easy")?;
        solve_grid_group(qqwing_intermediate_grids, "intermediate")?;
        solve_grid_group(qqwing_expert_grids, "expert")?;
        solve_grid_group(easy50_grids, "easy50")?;
        solve_grid_group(top95_grids, "top95")?;
        solve_grid_group(hardest_grids, "hardest")?;

        Ok(())
    }
}
