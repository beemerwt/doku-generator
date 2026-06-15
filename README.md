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

Parallel generation is enabled with `--workers`. The main thread keeps SQLite writes serialized, while worker threads generate accepted puzzles independently for each batch. Output is deterministic for a fixed `--seed`, `--difficulty`, `--symmetry`, `--batch-size`, and `--workers` value.

## Database

The database is created automatically if it does not exist. Puzzle strings are stored as 81 characters using digits `1-9` and `0` for blanks. Solution strings are 81 digits.

## Status

Implemented:

- deterministic solved-board generation
- brute-force uniqueness solver with MRV and bitmasks
- clue removal with optional rotational 180 symmetry
- human-style solver for naked singles, hidden singles, locked candidates, naked pairs, hidden pairs, naked triples, hidden triples, and X-Wing
- SQLite schema, indexes, WAL mode, and batch insertion
- CLI progress and summary output
- Rayon batch generation with serialized SQLite writes
- timing summary for solved-board generation, digging, uniqueness checks, rating, and database insertion

Puzzles are classified as `hard` only when implemented hard techniques such as naked triples, hidden triples, or X-Wing are detected.
