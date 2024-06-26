use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result};
use puzzles::sudoku::{self, Board};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

fn solve_set(name: &str, grids: Vec<Board>, solutions_dir: impl AsRef<Path>) -> Result<(u32, u32)> {
    let solution_path = solutions_dir.as_ref().join(name).with_extension("txt");
    let mut solution_file = File::create(&solution_path)
        .with_context(|| format!("Failed to create solution file '{solution_path:?}'."))?;
    let mut num_solved = 0;
    let mut num_set_steps = 0;
    let mut num_set_guesses = 0;
    for (index, grid) in grids.iter().enumerate() {
        let (solution, num_steps, num_guesses) = sudoku::solve(grid)
            .with_context(|| format!("Error while solving grid {index} in set {name}"))?;
        let solved = solution.validate().with_context(|| {
            format!(
                "Error validating solution for grid {index} in set {name}.\nSolution:\n{solution}Original board:\n{grid}"
            )
        })?.finished();
        if solved {
            num_solved += 1;
            num_set_steps += num_steps;
            num_set_guesses += num_guesses;
        }
        let solution_line = solution.to_pretty_string(Board::format_line, '.')?;
        writeln!(solution_file, "{solution_line},{solved}")
            .with_context(|| format!("Failed to write solution for grid {index} in set {name}."))?;
    }
    let num_grids = grids.len();

    let percentage = num_solved as f64 / num_grids as f64 * 100.0;
    println!("Solved {num_solved}/{num_grids} ({percentage:.0}%) {name} grids with {num_set_steps} steps and {num_set_guesses} guesses.",);
    Ok((num_set_steps, num_set_guesses))
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
            "insane",
            "blank",
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

        let start_time = Instant::now();
        let (num_total_steps, num_total_guesses) = sets
            .into_par_iter()
            .map(|(name, grids)| solve_set(name, grids, solutions_dir.as_path()).unwrap())
            .reduce(
                || (0, 0),
                |(total_steps, total_guesses), (set_steps, set_guesses)| {
                    (total_steps + set_steps, total_guesses + set_guesses)
                },
            );
        let elapsed = start_time.elapsed();
        println!("{num_total_steps} total steps and {num_total_guesses} guesses used on successful solutions");
        println!(
            "Total time: {}s {}ms",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );

        Ok(())
    }
}
