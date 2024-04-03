use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use puzzles::camping::{self, Map, MaybeTransposedMap};

#[derive(Clone, Debug, Args)]
pub struct Camping {
    map: Option<String>,
}

impl Camping {
    pub fn run(self) -> Result<()> {
        let camping_dir = PathBuf::from("data/camping");
        let maps_dir = camping_dir.join("maps");
        let output_dir = camping_dir.join("solutions");

        let maps = if let Some(map_name) = self.map {
            vec![(
                map_name.clone(),
                Map::from_file(maps_dir.join(&map_name).with_extension("txt"))?,
            )]
        } else {
            fs::read_dir(maps_dir)?
                .flat_map(|entry| {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => return Some(Err(err.into())),
                    };
                    let file_type = match entry.file_type() {
                        Ok(file_type) => file_type,
                        Err(err) => return Some(Err(err.into())),
                    };
                    if file_type.is_file()
                        && entry
                            .path()
                            .extension()
                            .and_then(OsStr::to_str)
                            .is_some_and(|ext| ext == "txt")
                    {
                        let map_name = entry.file_name().to_string_lossy().to_string();
                        let map = match Map::from_file(entry.path()) {
                            Ok(map) => map,
                            Err(err) => return Some(Err(err)),
                        };
                        Some(Ok((map_name, map)))
                    } else {
                        None
                    }
                })
                .collect::<Result<_>>()?
        };
        for (map_name, map) in maps {
            match camping::solve(&map) {
                Ok(Some(solution)) => {
                    match map.is_valid() {
                        Ok(()) => {}
                        Err(err) => {
                            eprintln!("Error while validating solution to '{map_name}': {err}");
                            continue;
                        }
                    }
                    let mut file = File::create(output_dir.join(&map_name).with_extension("txt"))?;
                    write!(file, "{solution}")?;
                    println!("Solution for '{map_name}' found and written to file.");
                }
                Ok(None) => println!("No solution found for '{map_name}'."),
                Err(err) => eprintln!("Error while solving '{map}': {err}"),
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
