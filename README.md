# doku-generator

Offline Sudoku puzzle generator for the Doku website.

It generates valid 9x9 Sudoku solutions, digs uniquely-solvable puzzles, rates them with a human-style solver, and stores accepted puzzles in SQLite with auto-incrementing IDs.

## Usage

```bash
cargo run --release -- generate \
  --database ./doku.sqlite \
  --difficulty easy \
  --count 1000 \
  --symmetry rotational180 \
  --batch-size 500
```

```bash
cargo run --release -- generate \
  --database ./doku.sqlite \
  --difficulty mixed \
  --count 100000 \
  --workers 8
```

The first implementation is single-threaded even when `--workers` is provided. Deterministic output is guaranteed for `--workers 1` with a fixed `--seed`.

## Database

The database is created automatically if it does not exist. Puzzle strings are stored as 81 characters using digits `1-9` and `.` for blanks. Solution strings are 81 digits.

## Status

Implemented:

- deterministic solved-board generation
- brute-force uniqueness solver with MRV and bitmasks
- clue removal with optional rotational 180 symmetry
- human-style solver for naked singles, hidden singles, locked candidates, naked pairs, and hidden pairs
- SQLite schema, indexes, WAL mode, and batch insertion
- CLI progress and summary output

Hard techniques (`NakedTriple`, `HiddenTriple`, `XWing`) are represented in metadata but not yet fully implemented, so generated puzzles are not classified as `hard` unless those techniques are later detected by real implementations.
