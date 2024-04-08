use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn insane(c: &mut Criterion) {
    let board_line = include_str!("../data/sudoku/grids/insane.txt");
    let board = puzzles::sudoku::Board::from_line(board_line, '.').unwrap();

    c.bench_with_input(BenchmarkId::new("solve", "insane"), &board, |b, board| {
        b.iter(|| puzzles::sudoku::solve(board).unwrap())
    });
}

criterion_group!(benches, insane);
criterion_main!(benches);
