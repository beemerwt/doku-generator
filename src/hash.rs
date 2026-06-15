use crate::board::Board;

pub fn puzzle_hash(puzzle: &Board, solution: &Board) -> anyhow::Result<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(puzzle.to_str81().as_bytes());
    hasher.update(solution.solution_str81()?.as_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}
