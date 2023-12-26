use std::{fs, path};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use ndarray::{Array1, Array2, Axis};
use serde::{Deserialize, Serialize};

use crate::location::Location;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Tree,
    Tent,
    Free,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    tiles: Array2<Tile>,
    row_requirements: Array1<usize>,
    col_requirements: Array1<usize>,
}

impl Map {
    pub fn new(
        tiles: Array2<Tile>,
        row_requirements: Array1<usize>,
        col_requirements: Array1<usize>,
    ) -> Self {
        assert_eq!(tiles.shape()[0], row_requirements.len());
        assert_eq!(tiles.shape()[1], col_requirements.len());
        Self {
            tiles,
            row_requirements,
            col_requirements,
        }
    }

    pub fn parse(string: impl AsRef<str>) -> Result<Self> {
        let string = string.as_ref();
        let mut lines = string.lines();
        let line = lines.next().context("No first line.")?;
        let (height, width): (&str, &str) = line.split(',').collect_tuple().with_context(|| {
            format!("Expected two integers separated by a comma. Got '{line}'.")
        })?;
        let height = height
            .parse::<usize>()
            .with_context(|| format!("Expected a positive integer height. Got '{height}'.",))?;
        let width = width
            .parse::<usize>()
            .with_context(|| format!("Expected a positive integer width. Got '{width}'.",))?;
        let line = lines.next().context("No second line.")?;
        let row_requirements = line
            .split(',')
            .map(|s| s.parse::<usize>())
            .collect::<Result<Array1<_>, _>>()
            .with_context(|| {
                format!(
                    "Expected {height} non-negative integers separated by commas. Got '{line}'.",
                )
            })?;
        if row_requirements.len() != height {
            return Err(anyhow::anyhow!(
                "Expected {height} non-negative integers separated by commas. Got {len} integers.",
                len = row_requirements.len()
            ));
        }
        let line = lines.next().context("No third line.")?;
        let col_requirements =
            line.split(',')
                .map(|s| s.parse::<usize>())
                .collect::<Result<Array1<_>, _>>()
                .with_context(|| {
                    format!(
                        "Expected {width} non-negative integers separated by commas. Got '{line}'.",
                    )
                })?;
        if col_requirements.len() != width {
            return Err(anyhow::anyhow!(
                "Expected {width} non-negative integers separated by commas. Got {len} integers.",
                len = col_requirements.len()
            ));
        }
        let x = lines
            .flat_map(|line| {
                line.chars().map(|c| match c {
                    'T' => Ok(Tile::Tree),
                    'X' => Ok(Tile::Tent),
                    ' ' => Ok(Tile::Free),
                    '#' => Ok(Tile::Blocked),
                    _ => Err(anyhow::anyhow!(
                        "Expected 'T', 'X', ' ', or '#'. Got '{c}'.",
                    )),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let tiles = Array2::from_shape_vec((height, width), x).with_context(|| {
            format!("Dimensions of map must match dimensions given at start of file.")
        })?;

        Ok(Self {
            tiles,
            row_requirements,
            col_requirements,
        })
    }

    pub fn from_file(path: impl AsRef<path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let string = fs::read_to_string(path)?;
        Self::parse(string)
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        let (height, width) = self.dim();
        string.push_str(&format!("{height},{width}\n"));
        string.push_str(&format!(
            "{}\n",
            self.row_requirements()
                .iter()
                .map(|&x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
        string.push_str(&format!(
            "{}\n",
            self.col_requirements()
                .iter()
                .map(|&x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
        string.push_str(&format!(
            "{}\n",
            self.tiles()
                .axis_iter(Axis(0))
                .map(|row| {
                    row.iter()
                        .map(|&t| match t {
                            Tile::Tree => 'T',
                            Tile::Tent => 'X',
                            Tile::Free => ' ',
                            Tile::Blocked => '#',
                        })
                        .join("")
                })
                .join("\n")
        ));
        string
    }

    pub fn dim(&self) -> (usize, usize) {
        self.tiles().dim()
    }

    pub fn height(&self) -> usize {
        self.tiles().shape()[0]
    }

    pub fn width(&self) -> usize {
        self.tiles().shape()[1]
    }

    pub fn in_bounds(&self, location: Location) -> bool {
        let (height, width) = self.dim();
        location.row < height && location.col < width
    }

    pub fn tiles(&self) -> &Array2<Tile> {
        &self.tiles
    }

    pub fn row_requirements(&self) -> &Array1<usize> {
        &self.row_requirements
    }

    pub fn col_requirements(&self) -> &Array1<usize> {
        &self.col_requirements
    }

    pub fn get(&self, location: Location) -> Option<Tile> {
        self.tiles.get((location.row, location.col)).copied()
    }

    pub fn adjacents(&self, location: Location) -> [Option<(Location, Tile)>; 4] {
        location
            .adjacents(self.dim())
            .map(move |loc| loc.map(|loc| (loc, self.get(loc).unwrap())))
    }

    pub fn neighbors(&self, location: Location) -> [Option<(Location, Tile)>; 8] {
        location
            .neighbors(self.dim())
            .map(move |loc| loc.map(|loc| (loc, self.get(loc).unwrap())))
    }

    pub fn is_valid(&self) -> bool {
        // RULES:
        // 1. Each row and column must have no more than the correct number of tents and enough free spaces to reach the required amount.
        // 2. Tents cannot be adjacent to each other, neither horizontally, vertically, nor diagonally.
        // 3. Tents must be placed adjacent to trees, horizontally and vertically.

        for (i, row) in self.tiles().axis_iter(Axis(0)).enumerate() {
            let requirement = self.row_requirements()[i];
            let num_tents = row.iter().filter(|&&t| t == Tile::Tent).count();
            let num_poss_tents = row
                .iter()
                .filter(|&&t| t == Tile::Free || t == Tile::Tent)
                .count();
            if !(num_tents <= requirement && num_poss_tents >= requirement) {
                return false; // Either too many tents, or too few possible tents.
            }
        }

        for (i, col) in self.tiles().axis_iter(Axis(1)).enumerate() {
            let requirement = self.col_requirements()[i];
            let num_tents = col.iter().filter(|&&t| t == Tile::Tent).count();
            let num_poss_tents = col
                .iter()
                .filter(|&&t| t == Tile::Free || t == Tile::Tent)
                .count();
            if !(num_tents <= requirement && num_poss_tents >= requirement) {
                return false; // Either too many tents, or too few possible tents.
            }
        }

        // Iterate over all tiles
        for ((row, col), &tile) in self.tiles().indexed_iter() {
            let loc = Location::new(row, col);
            match tile {
                Tile::Tree => {}
                Tile::Tent => {
                    if !self
                        .adjacents(loc)
                        .into_iter()
                        .filter_map(|x| x)
                        .any(|(_, t)| t == Tile::Tree)
                    {
                        return false; // No adjacent trees.
                    } else if self
                        .neighbors(loc)
                        .into_iter()
                        .filter_map(|x| x)
                        .any(|(_, t)| t == Tile::Tent)
                    {
                        return false; // Adjacent tent.
                    }
                }
                Tile::Free => {}
                Tile::Blocked => {}
            }
        }

        true
    }

    pub fn is_complete(&self) -> bool {
        // RULES:
        // 1. No free tiles exist.
        // 2. Map must be valid.

        self.tiles().iter().all(|&t| t != Tile::Free) && self.is_valid()
    }

    pub fn add_tent(mut self, location: Location) -> Result<Self> {
        if !(self.in_bounds(location)) {
            bail!("Cannot add tent to invalid location. Location out of bounds.");
        }
        if self.get(location) != Some(Tile::Free) {
            bail!("Cannot add tent to invalid location. Location is not free.");
        }
        self.tiles[(location.row, location.col)] = Tile::Tent;
        Ok(self)
    }

    pub fn add_blocked(mut self, location: Location) -> Result<Self> {
        if !(self.in_bounds(location)) {
            bail!("Cannot add blocked to invalid location. Location out of bounds.");
        }
        if self.get(location) != Some(Tile::Free) {
            bail!("Cannot add blocked to invalid location. Location is not free.");
        }
        self.tiles[(location.row, location.col)] = Tile::Blocked;
        Ok(self)
    }
}
