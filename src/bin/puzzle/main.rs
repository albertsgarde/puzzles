mod camping;
mod sudoku;

use anyhow::Result;
use camping::Camping;
use clap::{Parser, Subcommand};
use sudoku::Sudoku;

#[derive(Clone, Debug, Subcommand)]
pub enum Game {
    Camping(Camping),
    Sudoku(Sudoku),
}

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    game: Game,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.game {
            Game::Camping(camping) => camping.run()?,
            Game::Sudoku(sudoku) => sudoku.run()?,
        }
        Ok(())
    }
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
