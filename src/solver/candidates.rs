use crate::board::Board;

pub type CandidateMask = u16;

pub const ALL_DIGITS_MASK: CandidateMask = 0x01ff;

pub fn digit_bit(digit: u8) -> u16 {
    1u16 << (digit - 1)
}

pub fn mask_contains(mask: u16, digit: u8) -> bool {
    mask & digit_bit(digit) != 0
}

pub fn mask_count(mask: u16) -> u32 {
    mask.count_ones()
}

pub fn mask_single_digit(mask: u16) -> Option<u8> {
    if mask_count(mask) == 1 {
        Some(mask.trailing_zeros() as u8 + 1)
    } else {
        None
    }
}

pub fn row_of(index: usize) -> usize {
    index / 9
}

pub fn col_of(index: usize) -> usize {
    index % 9
}

pub fn box_of(index: usize) -> usize {
    (row_of(index) / 3) * 3 + col_of(index) / 3
}

#[allow(dead_code)]
pub fn peers(index: usize) -> Vec<usize> {
    let row = row_of(index);
    let col = col_of(index);
    let box_row = row / 3 * 3;
    let box_col = col / 3 * 3;
    let mut result = Vec::with_capacity(20);

    for c in 0..9 {
        let peer = row * 9 + c;
        if peer != index && !result.contains(&peer) {
            result.push(peer);
        }
    }
    for r in 0..9 {
        let peer = r * 9 + col;
        if peer != index && !result.contains(&peer) {
            result.push(peer);
        }
    }
    for r in box_row..box_row + 3 {
        for c in box_col..box_col + 3 {
            let peer = r * 9 + c;
            if peer != index && !result.contains(&peer) {
                result.push(peer);
            }
        }
    }
    result
}

pub fn units() -> Vec<[usize; 9]> {
    let mut units = Vec::with_capacity(27);
    for row in 0..9 {
        let mut unit = [0; 9];
        for (col, cell) in unit.iter_mut().enumerate() {
            *cell = row * 9 + col;
        }
        units.push(unit);
    }
    for col in 0..9 {
        let mut unit = [0; 9];
        for (row, cell) in unit.iter_mut().enumerate() {
            *cell = row * 9 + col;
        }
        units.push(unit);
    }
    for box_idx in 0..9 {
        let mut unit = [0; 9];
        let box_row = box_idx / 3 * 3;
        let box_col = box_idx % 3 * 3;
        let mut cursor = 0;
        for row in box_row..box_row + 3 {
            for col in box_col..box_col + 3 {
                unit[cursor] = row * 9 + col;
                cursor += 1;
            }
        }
        units.push(unit);
    }
    units
}

pub fn board_masks(board: &Board) -> Option<([u16; 9], [u16; 9], [u16; 9])> {
    let mut rows = [0u16; 9];
    let mut cols = [0u16; 9];
    let mut boxes = [0u16; 9];

    for index in 0..81 {
        let digit = board.get_index(index);
        if digit == 0 {
            continue;
        }
        if !(1..=9).contains(&digit) {
            return None;
        }
        let bit = digit_bit(digit);
        let row = row_of(index);
        let col = col_of(index);
        let box_idx = box_of(index);
        if rows[row] & bit != 0 || cols[col] & bit != 0 || boxes[box_idx] & bit != 0 {
            return None;
        }
        rows[row] |= bit;
        cols[col] |= bit;
        boxes[box_idx] |= bit;
    }

    Some((rows, cols, boxes))
}

pub fn candidates_for(index: usize, rows: &[u16; 9], cols: &[u16; 9], boxes: &[u16; 9]) -> u16 {
    let used = rows[row_of(index)] | cols[col_of(index)] | boxes[box_of(index)];
    ALL_DIGITS_MASK & !used
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_helpers_work() {
        let mask = digit_bit(3) | digit_bit(8);
        assert!(mask_contains(mask, 3));
        assert!(!mask_contains(mask, 4));
        assert_eq!(mask_count(mask), 2);
        assert_eq!(mask_single_digit(digit_bit(9)), Some(9));
        assert_eq!(mask_single_digit(mask), None);
    }
}
