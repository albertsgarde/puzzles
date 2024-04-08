use puzzles::sudoku::{solve, Board};

pub fn main() {
    let board_line = include_str!("../../data/sudoku/grids/insane.txt");
    let board = Board::from_line(board_line, '.').unwrap();
    solve(&board).unwrap();
}
