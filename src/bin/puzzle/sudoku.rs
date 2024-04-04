use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use puzzles::sudoku::{self, Board};

fn data_dir() -> PathBuf {
    PathBuf::from("data/sudoku")
}

fn output_dir() -> PathBuf {
    PathBuf::from("output/sudoku")
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

#[derive(Clone, Debug, clap::Args)]
pub struct Sudoku {}

impl Sudoku {
    pub fn run(self) -> Result<()> {
        let set_names = [
            "qqwing_simple",
            "qqwing_easy",
            "qqwing_intermediate",
            "qqwing_expert",
            "easy50",
            "top95",
            "hardest",
        ];

        let grid_dir = data_dir().join("grids");

        let sets: Vec<(&str, Vec<Board>)> = set_names
            .iter()
            .map(|&name| {
                load_grid_file(grid_dir.join(name).with_extension("txt"))
                    .with_context(|| format!("Error loading grid set {name}"))
                    .map(|grids| (name, grids))
            })
            .collect::<Result<_>>()?;

        let output_dir = output_dir();
        let solutions_dir = output_dir.join("solutions");
        fs::create_dir_all(&solutions_dir).with_context(|| {
            format!("Failed to create solutions directory '{solutions_dir:?}'.")
        })?;

        for (name, grids) in sets {
            let solution_path = solutions_dir.join(name).with_extension("txt");
            let mut solution_file = File::create(&solution_path)
                .with_context(|| format!("Failed to create solution file '{solution_path:?}'."))?;
            let mut num_solved = 0;
            for (index, grid) in grids.iter().enumerate() {
                let solution = sudoku::solve(grid)
                    .with_context(|| format!("Error while solving grid {index} in set {name}"))?;
                let solved = solution.validate().with_context(|| {
                    format!(
                        "Error validating solution for grid {index} in set {name}.\nSolution:\n{solution}Original board:\n{grid}"
                    )
                })?.finished();
                if solved {
                    num_solved += 1;
                }
                let solution_line = solution.to_pretty_string(Board::format_line, '.')?;
                writeln!(solution_file, "{solution_line},{solved}").with_context(|| {
                    format!("Failed to write solution for grid {index} in set {name}.")
                })?;
            }
            let num_grids = grids.len();

            let percentage = num_solved as f64 / num_grids as f64 * 100.0;
            println!("Solved {num_solved}/{num_grids} ({percentage:.0}%) {name} grids.",);
        }

        Ok(())
    }
}
