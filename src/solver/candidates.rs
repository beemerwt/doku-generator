use crate::board::Board;

pub type CandidateMask = u16;

pub const ALL_DIGITS_MASK: CandidateMask = 0x01ff;
pub const ROW_INDEX: [usize; 81] = build_row_index();
pub const COL_INDEX: [usize; 81] = build_col_index();
pub const BOX_INDEX: [usize; 81] = build_box_index();
pub const UNITS: [[usize; 9]; 27] = build_units();
#[allow(dead_code)]
pub const PEERS: [[usize; 20]; 81] = build_peers();

const fn build_row_index() -> [usize; 81] {
    let mut rows = [0; 81];
    let mut index = 0;
    while index < 81 {
        rows[index] = index / 9;
        index += 1;
    }
    rows
}

const fn build_col_index() -> [usize; 81] {
    let mut cols = [0; 81];
    let mut index = 0;
    while index < 81 {
        cols[index] = index % 9;
        index += 1;
    }
    cols
}

const fn build_box_index() -> [usize; 81] {
    let mut boxes = [0; 81];
    let mut index = 0;
    while index < 81 {
        let row = index / 9;
        let col = index % 9;
        boxes[index] = (row / 3) * 3 + col / 3;
        index += 1;
    }
    boxes
}

const fn build_units() -> [[usize; 9]; 27] {
    let mut units = [[0; 9]; 27];
    let mut row = 0;
    while row < 9 {
        let mut col = 0;
        while col < 9 {
            units[row][col] = row * 9 + col;
            col += 1;
        }
        row += 1;
    }

    let mut col = 0;
    while col < 9 {
        let mut row = 0;
        while row < 9 {
            units[9 + col][row] = row * 9 + col;
            row += 1;
        }
        col += 1;
    }

    let mut box_idx = 0;
    while box_idx < 9 {
        let box_row = (box_idx / 3) * 3;
        let box_col = (box_idx % 3) * 3;
        let mut cursor = 0;
        let mut row_offset = 0;
        while row_offset < 3 {
            let mut col_offset = 0;
            while col_offset < 3 {
                units[18 + box_idx][cursor] = (box_row + row_offset) * 9 + box_col + col_offset;
                cursor += 1;
                col_offset += 1;
            }
            row_offset += 1;
        }
        box_idx += 1;
    }

    units
}

const fn contains_prefix(values: &[usize; 20], len: usize, value: usize) -> bool {
    let mut index = 0;
    while index < len {
        if values[index] == value {
            return true;
        }
        index += 1;
    }
    false
}

const fn build_peers() -> [[usize; 20]; 81] {
    let mut peers = [[0; 20]; 81];
    let mut index = 0;
    while index < 81 {
        let row = index / 9;
        let col = index % 9;
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        let mut len = 0;

        let mut c = 0;
        while c < 9 {
            let peer = row * 9 + c;
            if peer != index && !contains_prefix(&peers[index], len, peer) {
                peers[index][len] = peer;
                len += 1;
            }
            c += 1;
        }

        let mut r = 0;
        while r < 9 {
            let peer = r * 9 + col;
            if peer != index && !contains_prefix(&peers[index], len, peer) {
                peers[index][len] = peer;
                len += 1;
            }
            r += 1;
        }

        let mut row_offset = 0;
        while row_offset < 3 {
            let mut col_offset = 0;
            while col_offset < 3 {
                let peer = (box_row + row_offset) * 9 + box_col + col_offset;
                if peer != index && !contains_prefix(&peers[index], len, peer) {
                    peers[index][len] = peer;
                    len += 1;
                }
                col_offset += 1;
            }
            row_offset += 1;
        }

        index += 1;
    }
    peers
}

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
    ROW_INDEX[index]
}

pub fn col_of(index: usize) -> usize {
    COL_INDEX[index]
}

pub fn box_of(index: usize) -> usize {
    BOX_INDEX[index]
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

#[allow(dead_code)]
pub fn units() -> Vec<[usize; 9]> {
    UNITS.to_vec()
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
