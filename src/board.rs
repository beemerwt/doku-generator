use anyhow::{anyhow, bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    cells: [u8; 81],
}

impl Board {
    pub fn empty() -> Self {
        Self { cells: [0; 81] }
    }

    #[allow(dead_code)]
    pub fn from_str81(input: &str) -> Result<Self> {
        if input.len() != 81 {
            bail!("board must be exactly 81 characters");
        }

        let mut board = Self::empty();
        for (idx, byte) in input.bytes().enumerate() {
            board.cells[idx] = match byte {
                b'0' => 0,
                b'1'..=b'9' => byte - b'0',
                _ => bail!("invalid board character at index {idx}"),
            };
        }
        Ok(board)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_str81(&self) -> String {
        self.cells
            .iter()
            .map(|&value| match value {
                0 => '0',
                1..=9 => char::from(b'0' + value),
                _ => '?',
            })
            .collect()
    }

    pub fn solution_str81(&self) -> Result<String> {
        if !self.is_complete() {
            bail!("solution board is not complete");
        }
        Ok(self
            .cells
            .iter()
            .map(|&value| char::from(b'0' + value))
            .collect())
    }

    #[allow(dead_code)]
    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.cells[row * 9 + col]
    }

    #[allow(dead_code)]
    pub fn set(&mut self, row: usize, col: usize, value: u8) {
        self.cells[row * 9 + col] = value;
    }

    pub fn get_index(&self, index: usize) -> u8 {
        self.cells[index]
    }

    pub fn set_index(&mut self, index: usize, value: u8) {
        self.cells[index] = value;
    }

    pub fn clue_count(&self) -> usize {
        self.cells.iter().filter(|&&value| value != 0).count()
    }

    pub fn is_complete(&self) -> bool {
        self.cells.iter().all(|&value| (1..=9).contains(&value))
    }

    pub fn cells(&self) -> &[u8; 81] {
        &self.cells
    }

    #[allow(dead_code)]
    pub fn validate_filled_consistency(&self) -> Result<()> {
        for index in 0..81 {
            let value = self.cells[index];
            if value == 0 {
                continue;
            }
            let row = index / 9;
            let col = index % 9;
            for other_col in 0..9 {
                let other = row * 9 + other_col;
                if other != index && self.cells[other] == value {
                    return Err(anyhow!("duplicate digit in row {}", row + 1));
                }
            }
            for other_row in 0..9 {
                let other = other_row * 9 + col;
                if other != index && self.cells[other] == value {
                    return Err(anyhow!("duplicate digit in column {}", col + 1));
                }
            }
            let box_row = row / 3 * 3;
            let box_col = col / 3 * 3;
            for r in box_row..box_row + 3 {
                for c in box_col..box_col + 3 {
                    let other = r * 9 + c;
                    if other != index && self.cells[other] == value {
                        return Err(anyhow!("duplicate digit in box"));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_serializes() {
        let input = "0".repeat(81);
        let board = Board::from_str81(&input).unwrap();
        assert_eq!(board.to_str81(), input);
        assert_eq!(board.clue_count(), 0);
    }

    #[test]
    fn rejects_bad_length() {
        assert!(Board::from_str81("0").is_err());
    }

    #[test]
    fn rejects_dot_blanks() {
        assert!(Board::from_str81(&".".repeat(81)).is_err());
    }
}
