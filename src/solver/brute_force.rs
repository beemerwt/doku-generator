use crate::{
    board::Board,
    solver::candidates::{board_masks, candidates_for, digit_bit},
};

pub fn count_solutions(board: &Board, limit: usize) -> usize {
    if limit == 0 {
        return 0;
    }
    let Some((mut rows, mut cols, mut boxes)) = board_masks(board) else {
        return 0;
    };
    let mut cells = *board.cells();
    let mut count = 0;
    solve_recursive(
        &mut cells, &mut rows, &mut cols, &mut boxes, limit, &mut count,
    );
    count
}

pub fn has_unique_solution(board: &Board) -> bool {
    count_solutions(board, 2) == 1
}

fn solve_recursive(
    cells: &mut [u8; 81],
    rows: &mut [u16; 9],
    cols: &mut [u16; 9],
    boxes: &mut [u16; 9],
    limit: usize,
    count: &mut usize,
) {
    if *count >= limit {
        return;
    }

    let mut best_index = None;
    let mut best_mask = 0u16;
    let mut best_count = 10u32;

    for (index, cell) in cells.iter().enumerate().take(81) {
        if *cell != 0 {
            continue;
        }
        let mask = candidates_for(index, rows, cols, boxes);
        let candidate_count = mask.count_ones();
        if candidate_count == 0 {
            return;
        }
        if candidate_count < best_count {
            best_index = Some(index);
            best_mask = mask;
            best_count = candidate_count;
            if candidate_count == 1 {
                break;
            }
        }
    }

    let Some(index) = best_index else {
        *count += 1;
        return;
    };

    let row = index / 9;
    let col = index % 9;
    let box_idx = (row / 3) * 3 + col / 3;
    let mut mask = best_mask;

    while mask != 0 {
        let bit = mask & (!mask + 1);
        let digit = bit.trailing_zeros() as u8 + 1;
        mask &= !bit;

        cells[index] = digit;
        rows[row] |= bit;
        cols[col] |= bit;
        boxes[box_idx] |= bit;

        solve_recursive(cells, rows, cols, boxes, limit, count);

        rows[row] &= !digit_bit(digit);
        cols[col] &= !digit_bit(digit);
        boxes[box_idx] &= !digit_bit(digit);
        cells[index] = 0;

        if *count >= limit {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::difficulty::Difficulty;
    use crate::generator::{
        puzzle::dig_puzzle, solved::generate_solved_board, symmetry::SymmetryMode,
    };

    #[test]
    fn solved_board_has_one_solution() {
        let board = generate_solved_board(42).unwrap();
        assert_eq!(count_solutions(&board, 2), 1);
    }

    #[test]
    fn invalid_board_has_zero_solutions() {
        let board = Board::from_str81(&format!("11{}", "0".repeat(79))).unwrap();
        assert_eq!(count_solutions(&board, 2), 0);
    }

    #[test]
    fn underconstrained_stops_at_two() {
        let board = Board::empty();
        assert_eq!(count_solutions(&board, 2), 2);
    }

    #[test]
    fn generated_puzzle_is_unique() {
        let solution = generate_solved_board(9).unwrap();
        let puzzle = dig_puzzle(&solution, 10, Difficulty::Easy, SymmetryMode::None).unwrap();
        assert!(has_unique_solution(&puzzle));
    }
}
