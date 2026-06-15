pub mod puzzle;
pub mod solved;
pub mod symmetry;

use anyhow::{bail, Result};
use rand::Rng;

use crate::{
    board::Board,
    difficulty::Difficulty,
    rating::rate_puzzle,
    solver::{brute_force::has_unique_solution, human::Technique},
    GENERATOR_VERSION,
};

use self::{puzzle::dig_puzzle, solved::generate_solved_board, symmetry::SymmetryMode};

#[derive(Debug, Clone)]
pub struct GeneratedPuzzle {
    pub puzzle: Board,
    pub solution: Board,
    pub difficulty: Difficulty,
    pub clue_count: usize,
    pub rating_score: u32,
    pub required_techniques: Vec<Technique>,
    pub solution_seed: u64,
    pub removal_seed: u64,
    pub generator_version: u32,
}

#[derive(Default, Debug, Clone)]
pub struct GenerationStats {
    pub candidates_attempted: u64,
    pub accepted_by_uniqueness: u64,
    pub accepted_by_difficulty: u64,
    pub solved_generation_nanos: u128,
    pub digging_nanos: u128,
    pub uniqueness_nanos: u128,
    pub rating_nanos: u128,
}

impl GenerationStats {
    pub fn merge(&mut self, other: &GenerationStats) {
        self.candidates_attempted += other.candidates_attempted;
        self.accepted_by_uniqueness += other.accepted_by_uniqueness;
        self.accepted_by_difficulty += other.accepted_by_difficulty;
        self.solved_generation_nanos += other.solved_generation_nanos;
        self.digging_nanos += other.digging_nanos;
        self.uniqueness_nanos += other.uniqueness_nanos;
        self.rating_nanos += other.rating_nanos;
    }
}

#[allow(dead_code)]
pub fn generate_one(
    requested: Difficulty,
    global_rng: &mut impl Rng,
    symmetry: SymmetryMode,
    stats: &mut GenerationStats,
) -> Result<GeneratedPuzzle> {
    generate_one_cancellable(requested, global_rng, symmetry, stats, || false)?
        .ok_or_else(|| anyhow::anyhow!("generation was interrupted"))
}

pub fn generate_one_cancellable(
    requested: Difficulty,
    global_rng: &mut impl Rng,
    symmetry: SymmetryMode,
    stats: &mut GenerationStats,
    should_stop: impl Fn() -> bool,
) -> Result<Option<GeneratedPuzzle>> {
    for _ in 0..100_000 {
        if should_stop() {
            return Ok(None);
        }
        stats.candidates_attempted += 1;
        let target = choose_target_difficulty(requested, global_rng);
        let solution_seed = global_rng.gen::<u64>();
        let removal_seed = global_rng.gen::<u64>();
        let started = std::time::Instant::now();
        let solution = generate_solved_board(solution_seed)?;
        stats.solved_generation_nanos += started.elapsed().as_nanos();
        if should_stop() {
            return Ok(None);
        }
        let started = std::time::Instant::now();
        let puzzle = dig_puzzle(&solution, removal_seed, target, symmetry)?;
        stats.digging_nanos += started.elapsed().as_nanos();
        if should_stop() {
            return Ok(None);
        }

        let started = std::time::Instant::now();
        if !has_unique_solution(&puzzle) {
            stats.uniqueness_nanos += started.elapsed().as_nanos();
            continue;
        }
        stats.uniqueness_nanos += started.elapsed().as_nanos();
        stats.accepted_by_uniqueness += 1;

        let started = std::time::Instant::now();
        let rating = rate_puzzle(&puzzle, target);
        stats.rating_nanos += started.elapsed().as_nanos();
        if rating.difficulty != target {
            continue;
        }
        stats.accepted_by_difficulty += 1;

        return Ok(Some(GeneratedPuzzle {
            puzzle,
            solution,
            difficulty: rating.difficulty,
            clue_count: puzzle.clue_count(),
            rating_score: rating.rating_score,
            required_techniques: rating.used_techniques,
            solution_seed,
            removal_seed,
            generator_version: GENERATOR_VERSION,
        }));
    }

    bail!("failed to generate a matching puzzle after many attempts")
}

fn choose_target_difficulty(requested: Difficulty, rng: &mut impl Rng) -> Difficulty {
    if requested != Difficulty::Mixed {
        return requested;
    }
    match rng.gen_range(0..100) {
        0..=24 => Difficulty::Easy,
        25..=54 => Difficulty::Medium,
        55..=79 => Difficulty::Hard,
        80..=94 => Difficulty::Expert,
        _ => Difficulty::Extreme,
    }
}
