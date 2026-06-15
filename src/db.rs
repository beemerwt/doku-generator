use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::{generator::GeneratedPuzzle, hash::puzzle_hash};

pub fn open_database(path: &str) -> Result<Connection> {
    let conn = Connection::open(path).with_context(|| format!("opening SQLite database {path}"))?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS puzzles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            puzzle TEXT NOT NULL CHECK(length(puzzle) = 81),
            solution TEXT NOT NULL CHECK(length(solution) = 81),

            difficulty TEXT NOT NULL,
            clue_count INTEGER NOT NULL,
            rating_score INTEGER NOT NULL,
            required_techniques TEXT NOT NULL,

            solution_seed INTEGER,
            removal_seed INTEGER,
            generator_version INTEGER NOT NULL,

            puzzle_hash TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_puzzles_difficulty ON puzzles(difficulty);
        CREATE INDEX IF NOT EXISTS idx_puzzles_clue_count ON puzzles(clue_count);
        CREATE INDEX IF NOT EXISTS idx_puzzles_rating_score ON puzzles(rating_score);
        "#,
    )?;
    Ok(())
}

pub fn close_database(conn: Connection) -> Result<()> {
    let checkpoint_result = conn.execute_batch(
        r#"
        PRAGMA optimize;
        PRAGMA wal_checkpoint(TRUNCATE);
        "#,
    );

    match conn.close() {
        Ok(()) => checkpoint_result.context("finalizing SQLite database before close"),
        Err((_conn, err)) => Err(err).context("closing SQLite database connection"),
    }
}

pub fn insert_puzzles(conn: &mut Connection, puzzles: &[GeneratedPuzzle]) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut inserted = 0usize;
    {
        let mut stmt = tx.prepare(
            r#"
            INSERT OR IGNORE INTO puzzles (
                puzzle,
                solution,
                difficulty,
                clue_count,
                rating_score,
                required_techniques,
                solution_seed,
                removal_seed,
                generator_version,
                puzzle_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )?;

        for puzzle in puzzles {
            let required_techniques = serde_json::to_string(&puzzle.required_techniques)?;
            let puzzle_text = puzzle.puzzle.to_str81();
            let solution_text = puzzle.solution.solution_str81()?;
            let hash = puzzle_hash(&puzzle.puzzle, &puzzle.solution)?;
            inserted += stmt.execute(params![
                puzzle_text,
                solution_text,
                puzzle.difficulty.to_string(),
                puzzle.clue_count as i64,
                puzzle.rating_score as i64,
                required_techniques,
                puzzle.solution_seed as i64,
                puzzle.removal_seed as i64,
                puzzle.generator_version as i64,
                hash,
            ])?;
        }
    }
    tx.commit()?;
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        difficulty::Difficulty,
        generator::{solved::generate_solved_board, GeneratedPuzzle},
        solver::human::Technique,
        GENERATOR_VERSION,
    };

    fn sample_generated() -> GeneratedPuzzle {
        let solution = generate_solved_board(123).unwrap();
        let mut puzzle = solution;
        puzzle.set_index(0, 0);
        GeneratedPuzzle {
            puzzle,
            solution,
            difficulty: Difficulty::Easy,
            clue_count: puzzle.clue_count(),
            rating_score: 10,
            required_techniques: vec![Technique::NakedSingle],
            solution_seed: 123,
            removal_seed: 456,
            generator_version: GENERATOR_VERSION,
        }
    }

    #[test]
    fn creates_schema() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'puzzles'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn duplicate_insert_is_ignored() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        let puzzle = sample_generated();
        assert_eq!(
            insert_puzzles(&mut conn, std::slice::from_ref(&puzzle)).unwrap(),
            1
        );
        assert_eq!(insert_puzzles(&mut conn, &[puzzle]).unwrap(), 0);
    }
}
