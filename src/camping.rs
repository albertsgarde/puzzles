mod map;
pub use map::{Map, MaybeTransposedMap, PlacementError, Tile, TransposedMap};
mod solver;
pub use solver::{presolve, solve, solve_step};
