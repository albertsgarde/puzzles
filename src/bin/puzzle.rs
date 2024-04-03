use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use puzzles::camping::{self, Map, MaybeTransposedMap};

#[derive(Clone, Debug, Args)]
pub struct Camping {
    map: String,
}

impl Camping {
    pub fn run(self) -> Result<()> {
        let camping_dir = PathBuf::from("data/camping");
        let maps_dir = camping_dir.join("maps");
        let output_dir = camping_dir.join("solutions");
        let map = Map::from_file(maps_dir.join(&self.map).with_extension("txt"))?;

        map.is_valid().unwrap();

        println!("{map}");
        let (solved_map, solved) = camping::solve(&map).expect("Error while solving.");
        if solved {
            println!("Solution:\n{solved_map}");
            let mut file = File::create(output_dir.join(&self.map).with_extension("txt"))?;
            write!(file, "{solved_map}")?;
            println!("Solution found and written to file.");
        } else {
            println!("Failed to find solution:\n{solved_map}");
            println!("No solution found.");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Game {
    Camping(Camping),
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
        }
        Ok(())
    }
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
