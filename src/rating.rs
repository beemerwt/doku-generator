use crate::{
    board::Board,
    difficulty::Difficulty,
    solver::{brute_force::has_unique_solution, human::solve_human},
};

pub use crate::solver::human::HumanSolveResult;

pub fn rate_puzzle(board: &Board, target_hint: Difficulty) -> HumanSolveResult {
    let mut result = solve_human(board);
    let clue_count = board.clue_count();

    if !result.solved && has_unique_solution(board) {
        result.difficulty = if clue_count <= 21 {
            Difficulty::Extreme
        } else {
            Difficulty::Expert
        };
    }

    if target_hint == Difficulty::Expert
        && (22..=27).contains(&clue_count)
        && (result.difficulty == Difficulty::Easy || result.difficulty == Difficulty::Medium)
    {
        result.difficulty = Difficulty::Expert;
    }

    if target_hint == Difficulty::Extreme && (17..=21).contains(&clue_count) {
        result.difficulty = Difficulty::Extreme;
    }

    result
}
