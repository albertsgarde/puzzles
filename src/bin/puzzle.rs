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
        println!("Performing presolve...");
        let mut map = camping::pre_solve(map);
        println!("{map}");
        for i in 0.. {
            println!("Step {i}...");
            let (new_map, changed) = camping::solve_step(map);
            map = new_map;
            if !changed {
                println!("No changes this step. Stopping.");
                break;
            }
            println!("{map}");
            if map.is_complete() {
                println!("Map is complete. Stopping.");
                let out_file = output_dir.join(&self.map).with_extension("txt");
                write!(File::create(out_file)?, "{map}")?;
                break;
            }
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
