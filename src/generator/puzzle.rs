use anyhow::{bail, Result};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{board::Board, difficulty::Difficulty, solver::brute_force::has_unique_solution};

use super::symmetry::SymmetryMode;

pub fn dig_puzzle(
    solution: &Board,
    removal_seed: u64,
    difficulty: Difficulty,
    symmetry: SymmetryMode,
) -> Result<Board> {
    let Some((min_clues, max_clues)) = difficulty.clue_range() else {
        bail!("cannot dig directly for mixed difficulty");
    };
    let mut rng = ChaCha8Rng::seed_from_u64(removal_seed);
    let mut puzzle = *solution;
    let mut positions = (0..81).collect::<Vec<_>>();
    positions.shuffle(&mut rng);

    for index in positions {
        if puzzle.clue_count() <= min_clues {
            break;
        }
        if puzzle.get_index(index) == 0 {
            continue;
        }

        let mut removed = vec![index];
        if symmetry == SymmetryMode::Rotational180 {
            let opposite = 80 - index;
            if opposite != index {
                removed.push(opposite);
            }
        }

        let previous = removed
            .iter()
            .map(|&idx| (idx, puzzle.get_index(idx)))
            .collect::<Vec<_>>();
        for &idx in &removed {
            puzzle.set_index(idx, 0);
        }

        if puzzle.clue_count() < min_clues || !has_unique_solution(&puzzle) {
            for (idx, value) in previous {
                puzzle.set_index(idx, value);
            }
        }
    }

    if puzzle.clue_count() > max_clues {
        // Retry once with a target-biased second pass by scanning cells in the existing order.
        for index in 0..81 {
            if puzzle.clue_count() <= max_clues {
                break;
            }
            if puzzle.get_index(index) == 0 {
                continue;
            }
            let opposite = 80 - index;
            let removed = if symmetry == SymmetryMode::Rotational180 && opposite != index {
                vec![index, opposite]
            } else {
                vec![index]
            };
            let previous = removed
                .iter()
                .map(|&idx| (idx, puzzle.get_index(idx)))
                .collect::<Vec<_>>();
            for &idx in &removed {
                puzzle.set_index(idx, 0);
            }
            if puzzle.clue_count() < min_clues || !has_unique_solution(&puzzle) {
                for (idx, value) in previous {
                    puzzle.set_index(idx, value);
                }
            }
        }
    }

    Ok(puzzle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::solved::generate_solved_board;

    #[test]
    fn digging_preserves_uniqueness() {
        let solution = generate_solved_board(11).unwrap();
        let puzzle = dig_puzzle(&solution, 12, Difficulty::Medium, SymmetryMode::None).unwrap();
        assert!(has_unique_solution(&puzzle));
    }

    #[test]
    fn rotational_symmetry_is_preserved() {
        let solution = generate_solved_board(13).unwrap();
        let puzzle =
            dig_puzzle(&solution, 14, Difficulty::Easy, SymmetryMode::Rotational180).unwrap();
        for idx in 0..81 {
            assert_eq!(puzzle.get_index(idx) == 0, puzzle.get_index(80 - idx) == 0);
        }
    }

    #[test]
    fn generated_strings_are_81_chars() {
        let solution = generate_solved_board(15).unwrap();
        let puzzle = dig_puzzle(&solution, 16, Difficulty::Easy, SymmetryMode::None).unwrap();
        assert_eq!(puzzle.to_str81().len(), 81);
        assert_eq!(solution.solution_str81().unwrap().len(), 81);
    }
}
