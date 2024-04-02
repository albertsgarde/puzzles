use std::{fmt::Display, fs, path};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use ndarray::{Array1, Array2, ArrayView2, Axis};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::location::Location;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Tree,
    Tent,
    Free,
    Blocked,
}

#[derive(Clone, Copy, Debug, Error)]
pub enum PlacementError {
    #[error("Location {0} is out of bounds.")]
    OutOfBounds(Location),
    #[error("Location {location} is not free. Tile is {tile:?}.")]
    NotFree { location: Location, tile: Tile },
}

#[derive(Clone, Copy, Debug, Error)]
pub enum InvalidMapError {
    #[error(
        "Too few placable tents in row {row_index}. {possible} possible, {required} required."
    )]
    TooFewPossibleTentsInRow {
        row_index: usize,
        possible: usize,
        required: usize,
    },
    #[error("Too many tents in row {row_index}. Placed {placed}, required {required}.")]
    TooManyTentsInRow {
        row_index: usize,
        placed: usize,
        required: usize,
    },
    #[error(
        "Too few placable tents in column {col_index}. {possible} possible, {required} required."
    )]
    TooFewPossibleTentsInCol {
        col_index: usize,
        possible: usize,
        required: usize,
    },
    #[error("Too many tents in column {col_index}. Placed {placed}, required {required}.")]
    TooManyTentsInCol {
        col_index: usize,
        placed: usize,
        required: usize,
    },
    #[error("Tent not adjacent to tree at {location}.")]
    TentNotAdjacentToTree { location: Location },
    #[error("Pair of neighbouring tents at locations {loc1} and {loc2}.")]
    NeighbouringTents { loc1: Location, loc2: Location },
}

pub trait MaybeTransposedMap: Sized {
    fn map(&self) -> &Map;
    fn dim(&self) -> (usize, usize);
    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn in_bounds(&self, location: Location) -> bool;
    fn tiles(&self) -> ArrayView2<Tile>;
    fn row_requirements(&self) -> &Array1<usize>;
    fn col_requirements(&self) -> &Array1<usize>;
    fn get(&self, location: Location) -> Option<Tile>;
    fn adjacents(&self, location: Location) -> [Option<(Location, Tile)>; 4];
    fn neighbors(&self, location: Location) -> [Option<(Location, Tile)>; 8];
    fn is_valid(&self) -> Result<(), InvalidMapError>;
    fn is_complete(&self) -> bool;
    fn add_tent(self, location: Location) -> Result<Self>;
    fn add_blocked(self, location: Location) -> Result<Self>;
    fn ref_add_tent(&mut self, location: Location) -> Result<(), PlacementError>;
    fn ref_add_blocked(&mut self, location: Location) -> Result<(), PlacementError>;
    fn num_possible_row_tents(&self, row_index: usize) -> usize;
    fn num_possible_col_tents(&self, col_index: usize) -> usize;
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
        let tiles = Array2::from_shape_vec((height, width), x)
            .with_context(|| "Dimensions of map must match dimensions given at start of file.")?;

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

    pub fn transpose(self) -> TransposedMap {
        TransposedMap { map: self }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (height, width) = self.dim();
        writeln!(f, "{height},{width}")?;
        writeln!(f, "{}", self.row_requirements().iter().join(","))?;
        writeln!(f, "{}", self.col_requirements().iter().join(","))?;
        writeln!(
            f,
            "{}",
            self.tiles()
                .axis_iter(Axis(0))
                .map(|row| row
                    .iter()
                    .map(|&t| match t {
                        Tile::Tree => 'T',
                        Tile::Tent => 'X',
                        Tile::Free => ' ',
                        Tile::Blocked => '#',
                    })
                    .join(""))
                .join("\n")
        )
    }
}

impl MaybeTransposedMap for Map {
    fn map(&self) -> &Map {
        self
    }

    fn dim(&self) -> (usize, usize) {
        self.tiles().dim()
    }

    fn height(&self) -> usize {
        self.tiles().shape()[0]
    }

    fn width(&self) -> usize {
        self.tiles().shape()[1]
    }

    fn in_bounds(&self, location: Location) -> bool {
        let (height, width) = self.dim();
        location.row < height && location.col < width
    }

    fn tiles(&self) -> ArrayView2<Tile> {
        self.tiles.view()
    }

    fn row_requirements(&self) -> &Array1<usize> {
        &self.row_requirements
    }

    fn col_requirements(&self) -> &Array1<usize> {
        &self.col_requirements
    }

    fn get(&self, location: Location) -> Option<Tile> {
        self.tiles.get((location.row, location.col)).copied()
    }

    fn adjacents(&self, location: Location) -> [Option<(Location, Tile)>; 4] {
        location
            .adjacents(self.dim())
            .map(move |loc| loc.map(|loc| (loc, self.get(loc).unwrap())))
    }

    fn neighbors(&self, location: Location) -> [Option<(Location, Tile)>; 8] {
        location.neighbors(self.dim()).map(move |loc| {
            loc.map(|loc| (loc, self.get(loc).unwrap_or_else(|| panic!("{:?}", loc))))
        })
    }

    fn is_valid(&self) -> Result<(), InvalidMapError> {
        // RULES:
        // 1. Each row and column must have no more than the correct number of tents and enough free spaces to reach the required amount.
        // 2. Tents cannot be adjacent to each other, neither horizontally, vertically, nor diagonally.
        // 3. Tents must be placed adjacent to trees, horizontally and vertically.

        for (row_index, row) in self.tiles().axis_iter(Axis(0)).enumerate() {
            let requirement = self.row_requirements()[row_index];
            let num_tents = row.iter().filter(|&&t| t == Tile::Tent).count();
            let num_poss_tents = row
                .iter()
                .filter(|&&t| t == Tile::Free || t == Tile::Tent)
                .count();
            if num_tents > requirement {
                return Err(InvalidMapError::TooManyTentsInRow {
                    row_index,
                    placed: num_tents,
                    required: requirement,
                });
            }
            if num_poss_tents < requirement {
                return Err(InvalidMapError::TooFewPossibleTentsInRow {
                    row_index,
                    possible: num_poss_tents,
                    required: requirement,
                });
            }
        }

        for (col_index, col) in self.tiles().axis_iter(Axis(1)).enumerate() {
            let requirement = self.col_requirements()[col_index];
            let num_tents = col.iter().filter(|&&t| t == Tile::Tent).count();
            let num_poss_tents = col
                .iter()
                .filter(|&&t| t == Tile::Free || t == Tile::Tent)
                .count();
            if num_tents > requirement {
                return Err(InvalidMapError::TooManyTentsInCol {
                    col_index,
                    placed: num_tents,
                    required: requirement,
                });
            }
            if num_poss_tents < requirement {
                return Err(InvalidMapError::TooFewPossibleTentsInCol {
                    col_index,
                    possible: num_poss_tents,
                    required: requirement,
                });
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
                        .flatten()
                        .any(|(_, t)| t == Tile::Tree)
                    {
                        return Err(InvalidMapError::TentNotAdjacentToTree { location: loc });
                    }
                    if let Some((other_loc, _tile)) = self
                        .neighbors(loc)
                        .into_iter()
                        .flatten()
                        .find(|&(_, t)| t == Tile::Tent)
                    {
                        return Err(InvalidMapError::NeighbouringTents {
                            loc1: loc,
                            loc2: other_loc,
                        });
                    }
                }
                Tile::Free => {}
                Tile::Blocked => {}
            }
        }

        Ok(())
    }

    fn is_complete(&self) -> bool {
        // RULES:
        // 1. No free tiles exist.
        // 2. Map must be valid.

        self.tiles().iter().all(|&t| t != Tile::Free) && self.is_valid().is_ok()
    }

    fn add_tent(mut self, location: Location) -> Result<Self> {
        if !(self.in_bounds(location)) {
            bail!("Cannot add tent to invalid location {location}. Location out of bounds.");
        }
        if self.get(location) != Some(Tile::Free) {
            bail!("Cannot add tent to invalid location {location}. Location is not free.");
        }
        self.tiles[(location.row, location.col)] = Tile::Tent;
        Ok(self)
    }

    fn ref_add_tent(&mut self, location: Location) -> Result<(), PlacementError> {
        if let Some(tile) = self.get(location) {
            if tile != Tile::Free {
                Err(PlacementError::NotFree { location, tile })
            } else {
                self.tiles[(location.row, location.col)] = Tile::Tent;
                Ok(())
            }
        } else {
            Err(PlacementError::OutOfBounds(location))
        }
    }

    fn add_blocked(mut self, location: Location) -> Result<Self> {
        if !(self.in_bounds(location)) {
            bail!("Cannot add blocked to invalid location {location}. Location out of bounds.");
        }
        if self.get(location) != Some(Tile::Free) {
            bail!("Cannot add blocked to invalid location {location}. Location is not free.");
        }
        self.tiles[(location.row, location.col)] = Tile::Blocked;
        Ok(self)
    }

    fn ref_add_blocked(&mut self, location: Location) -> Result<(), PlacementError> {
        if let Some(tile) = self.get(location) {
            if tile != Tile::Free {
                Err(PlacementError::NotFree { location, tile })
            } else {
                self.tiles[(location.row, location.col)] = Tile::Blocked;
                Ok(())
            }
        } else {
            Err(PlacementError::OutOfBounds(location))
        }
    }

    /// Number of tents that could possibly be added to this row using only information stored in this row.
    fn num_possible_row_tents(&self, row_index: usize) -> usize {
        let mut total = 0;
        let mut prev = false;
        for &tile in self.tiles().row(row_index) {
            if prev {
                prev = false;
            } else if tile == Tile::Free {
                total += 1;
                prev = true;
            }
        }
        total
    }

    fn num_possible_col_tents(&self, col_index: usize) -> usize {
        let mut total = 0;
        let mut prev = false;
        for &tile in self.tiles().column(col_index) {
            if prev {
                prev = false;
            } else if tile == Tile::Free {
                total += 1;
                prev = true;
            }
        }
        total
    }
}

pub struct TransposedMap {
    map: Map,
}

impl TransposedMap {
    pub fn untranspose(self) -> Map {
        self.map
    }
}

impl MaybeTransposedMap for TransposedMap {
    fn map(&self) -> &Map {
        &self.map
    }

    fn dim(&self) -> (usize, usize) {
        let (base_width, base_height) = self.map.dim();
        (base_height, base_width)
    }

    fn height(&self) -> usize {
        self.map.width()
    }

    fn width(&self) -> usize {
        self.map.height()
    }

    fn in_bounds(&self, location: Location) -> bool {
        self.map.in_bounds(location.transpose())
    }

    fn tiles(&self) -> ArrayView2<Tile> {
        let mut tiles = self.map.tiles();
        tiles.swap_axes(0, 1);
        tiles
    }

    fn row_requirements(&self) -> &Array1<usize> {
        self.map.col_requirements()
    }

    fn col_requirements(&self) -> &Array1<usize> {
        self.map.row_requirements()
    }

    fn get(&self, location: Location) -> Option<Tile> {
        self.map.get(location.transpose())
    }

    fn adjacents(&self, location: Location) -> [Option<(Location, Tile)>; 4] {
        self.map
            .adjacents(location.transpose())
            .map(|loc| loc.map(|(loc, t)| (loc.transpose(), t)))
    }

    fn neighbors(&self, location: Location) -> [Option<(Location, Tile)>; 8] {
        self.map
            .neighbors(location.transpose())
            .map(|loc| loc.map(|(loc, t)| (loc.transpose(), t)))
    }

    fn is_valid(&self) -> Result<(), InvalidMapError> {
        self.map.is_valid()
    }

    fn is_complete(&self) -> bool {
        self.map.is_complete()
    }

    fn add_tent(self, location: Location) -> Result<Self> {
        Ok(Self {
            map: self.map.add_tent(location.transpose())?,
        })
    }

    fn ref_add_tent(&mut self, location: Location) -> Result<(), PlacementError> {
        self.map.ref_add_tent(location.transpose())
    }

    fn add_blocked(self, location: Location) -> Result<Self> {
        Ok(Self {
            map: self.map.add_blocked(location.transpose())?,
        })
    }

    fn ref_add_blocked(&mut self, location: Location) -> Result<(), PlacementError> {
        self.map.ref_add_blocked(location.transpose())
    }

    fn num_possible_row_tents(&self, row_index: usize) -> usize {
        self.map.num_possible_col_tents(row_index)
    }

    fn num_possible_col_tents(&self, col_index: usize) -> usize {
        self.map.num_possible_row_tents(col_index)
    }
}
