use anyhow::{Context, Result};
use puzzles::sudoku::Board;

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

fn load_qqwing(postfix: impl AsRef<str>) -> Result<Vec<Board>> {
    let grid_dir = data_dir().join("grids");
    let file_path = grid_dir
        .join(format!("qqwing_{}", postfix.as_ref()))
        .with_extension("txt");
    let data_str = std::fs::read_to_string(file_path.as_path())
        .with_context(|| format!("Failed to read grid file '{file_path:?}'."))?;
    read_boards_from_lines(data_str.lines(), '.') // qqwing uses '.' for empty cells
}

#[derive(Clone, Debug, clap::Args)]
pub struct Sudoku {}

impl Sudoku {
    pub fn run(self) -> Result<()> {
        let qqwing_simple_grids =
            load_qqwing("simple").context("Error loading simple qqwing grids")?;
        let first_grid = qqwing_simple_grids.first().unwrap();
        println!("{first_grid}");
        Ok(())
    }
}
