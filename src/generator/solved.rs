use anyhow::Result;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    board::Board,
    solver::candidates::{board_masks, candidates_for},
};

pub fn generate_solved_board(seed: u64) -> Result<Board> {
    let mut board = Board::empty();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    fill_board(&mut board, &mut rng);
    Ok(board)
}

fn fill_board(board: &mut Board, rng: &mut ChaCha8Rng) -> bool {
    let Some((rows, cols, boxes)) = board_masks(board) else {
        return false;
    };

    let mut best_index = None;
    let mut best_mask = 0u16;
    let mut best_count = 10u32;

    for index in 0..81 {
        if board.get_index(index) != 0 {
            continue;
        }
        let mask = candidates_for(index, &rows, &cols, &boxes);
        let count = mask.count_ones();
        if count == 0 {
            return false;
        }
        if count < best_count {
            best_index = Some(index);
            best_mask = mask;
            best_count = count;
        }
    }

    let Some(index) = best_index else {
        return true;
    };

    let mut digits = Vec::with_capacity(best_count as usize);
    for digit in 1..=9 {
        if best_mask & (1 << (digit - 1)) != 0 {
            digits.push(digit);
        }
    }
    digits.shuffle(rng);

    for digit in digits {
        board.set_index(index, digit);
        if fill_board(board, rng) {
            return true;
        }
        board.set_index(index, 0);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_unit(values: impl Iterator<Item = u8>) -> Vec<u8> {
        let mut values = values.collect::<Vec<_>>();
        values.sort_unstable();
        values
    }

    #[test]
    fn solved_board_is_valid() {
        let board = generate_solved_board(123).unwrap();
        let expected = (1..=9).collect::<Vec<_>>();

        for row in 0..9 {
            assert_eq!(sorted_unit((0..9).map(|col| board.get(row, col))), expected);
        }
        for col in 0..9 {
            assert_eq!(sorted_unit((0..9).map(|row| board.get(row, col))), expected);
        }
        for box_row in (0..9).step_by(3) {
            for box_col in (0..9).step_by(3) {
                assert_eq!(
                    sorted_unit((box_row..box_row + 3).flat_map(|row| {
                        (box_col..box_col + 3).map(move |col| board.get(row, col))
                    })),
                    expected
                );
            }
        }
    }

    #[test]
    fn same_seed_same_solution() {
        assert_eq!(
            generate_solved_board(99).unwrap(),
            generate_solved_board(99).unwrap()
        );
    }
}
