use serde::Serialize;

use crate::{
    board::Board,
    difficulty::Difficulty,
    solver::candidates::{
        box_of, candidates_for, col_of, mask_contains, mask_count, mask_single_digit, row_of,
        units, ALL_DIGITS_MASK,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Technique {
    NakedSingle,
    HiddenSingle,
    LockedCandidate,
    NakedPair,
    HiddenPair,
    NakedTriple,
    HiddenTriple,
    XWing,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolveStep {
    pub technique: Technique,
    pub cell: Option<usize>,
    pub digit: Option<u8>,
    pub eliminated: Vec<(usize, u8)>,
}

#[derive(Debug, Clone)]
pub struct HumanSolveResult {
    pub solved: bool,
    #[allow(dead_code)]
    pub steps: Vec<SolveStep>,
    pub used_techniques: Vec<Technique>,
    pub rating_score: u32,
    pub difficulty: Difficulty,
}

#[derive(Clone)]
struct CandidateState {
    board: Board,
    masks: [u16; 81],
}

impl CandidateState {
    fn new(board: &Board) -> Option<Self> {
        let mut state = Self {
            board: *board,
            masks: [0; 81],
        };
        state.recompute()?;
        Some(state)
    }

    fn recompute(&mut self) -> Option<()> {
        let (rows, cols, boxes) = crate::solver::candidates::board_masks(&self.board)?;
        for index in 0..81 {
            self.masks[index] = if self.board.get_index(index) == 0 {
                candidates_for(index, &rows, &cols, &boxes)
            } else {
                0
            };
        }
        Some(())
    }

    fn place(&mut self, index: usize, digit: u8) -> bool {
        if self.board.get_index(index) != 0 {
            return false;
        }
        self.board.set_index(index, digit);
        self.recompute().is_some()
    }

    fn eliminate(&mut self, eliminations: &[(usize, u8)]) -> bool {
        let mut changed = false;
        for &(index, digit) in eliminations {
            if self.board.get_index(index) == 0 && mask_contains(self.masks[index], digit) {
                self.masks[index] &= !crate::solver::candidates::digit_bit(digit);
                changed = true;
            }
        }
        changed
    }
}

pub fn solve_human(board: &Board) -> HumanSolveResult {
    let Some(mut state) = CandidateState::new(board) else {
        return HumanSolveResult {
            solved: false,
            steps: Vec::new(),
            used_techniques: Vec::new(),
            rating_score: 0,
            difficulty: Difficulty::Expert,
        };
    };

    let mut steps = Vec::new();
    loop {
        if state.board.is_complete() {
            break;
        }
        if let Some(step) = apply_naked_single(&mut state) {
            steps.push(step);
        } else if let Some(step) = apply_hidden_single(&mut state) {
            steps.push(step);
        } else if let Some(step) = apply_locked_candidate(&mut state) {
            steps.push(step);
        } else if let Some(step) = apply_naked_pair(&mut state) {
            steps.push(step);
        } else if let Some(step) = apply_hidden_pair(&mut state) {
            steps.push(step);
        } else if let Some(step) = apply_naked_triple(&mut state) {
            steps.push(step);
        } else if let Some(step) = apply_hidden_triple(&mut state) {
            steps.push(step);
        } else {
            break;
        }
    }

    let used_techniques = ordered_used_techniques(&steps);
    let rating_score = steps
        .iter()
        .map(|step| technique_score(step.technique))
        .sum();
    let difficulty = classify_human_result(
        board.clue_count(),
        state.board.is_complete(),
        &used_techniques,
        rating_score,
    );

    HumanSolveResult {
        solved: state.board.is_complete(),
        steps,
        used_techniques,
        rating_score,
        difficulty,
    }
}

fn apply_naked_single(state: &mut CandidateState) -> Option<SolveStep> {
    for index in 0..81 {
        if state.board.get_index(index) == 0 {
            if let Some(digit) = mask_single_digit(state.masks[index]) {
                state.place(index, digit);
                return Some(SolveStep {
                    technique: Technique::NakedSingle,
                    cell: Some(index),
                    digit: Some(digit),
                    eliminated: Vec::new(),
                });
            }
        }
    }
    None
}

fn apply_hidden_single(state: &mut CandidateState) -> Option<SolveStep> {
    for unit in units() {
        for digit in 1..=9 {
            let mut found = None;
            let mut count = 0;
            for index in unit {
                if state.board.get_index(index) == 0 && mask_contains(state.masks[index], digit) {
                    found = Some(index);
                    count += 1;
                }
            }
            if count == 1 {
                let index = found?;
                state.place(index, digit);
                return Some(SolveStep {
                    technique: Technique::HiddenSingle,
                    cell: Some(index),
                    digit: Some(digit),
                    eliminated: Vec::new(),
                });
            }
        }
    }
    None
}

fn apply_locked_candidate(state: &mut CandidateState) -> Option<SolveStep> {
    for box_idx in 0..9 {
        let box_row = box_idx / 3 * 3;
        let box_col = box_idx % 3 * 3;
        for digit in 1..=9 {
            let mut locations = Vec::new();
            for row in box_row..box_row + 3 {
                for col in box_col..box_col + 3 {
                    let index = row * 9 + col;
                    if state.board.get_index(index) == 0 && mask_contains(state.masks[index], digit)
                    {
                        locations.push(index);
                    }
                }
            }
            if locations.len() < 2 {
                continue;
            }
            let same_row = locations
                .iter()
                .all(|&idx| row_of(idx) == row_of(locations[0]));
            let same_col = locations
                .iter()
                .all(|&idx| col_of(idx) == col_of(locations[0]));
            let mut eliminated = Vec::new();
            if same_row {
                let row = row_of(locations[0]);
                for col in 0..9 {
                    let index = row * 9 + col;
                    if box_of(index) != box_idx
                        && state.board.get_index(index) == 0
                        && mask_contains(state.masks[index], digit)
                    {
                        eliminated.push((index, digit));
                    }
                }
            }
            if same_col {
                let col = col_of(locations[0]);
                for row in 0..9 {
                    let index = row * 9 + col;
                    if box_of(index) != box_idx
                        && state.board.get_index(index) == 0
                        && mask_contains(state.masks[index], digit)
                    {
                        eliminated.push((index, digit));
                    }
                }
            }
            if !eliminated.is_empty() && state.eliminate(&eliminated) {
                return Some(SolveStep {
                    technique: Technique::LockedCandidate,
                    cell: None,
                    digit: Some(digit),
                    eliminated,
                });
            }
        }
    }
    None
}

fn apply_naked_pair(state: &mut CandidateState) -> Option<SolveStep> {
    for unit in units() {
        for a in 0..8 {
            let idx_a = unit[a];
            let mask = state.masks[idx_a];
            if state.board.get_index(idx_a) != 0 || mask_count(mask) != 2 {
                continue;
            }
            for &idx_b in unit.iter().skip(a + 1) {
                if state.board.get_index(idx_b) == 0 && state.masks[idx_b] == mask {
                    let mut eliminated = Vec::new();
                    for &idx in &unit {
                        if idx == idx_a || idx == idx_b || state.board.get_index(idx) != 0 {
                            continue;
                        }
                        for digit in 1..=9 {
                            if mask_contains(mask, digit) && mask_contains(state.masks[idx], digit)
                            {
                                eliminated.push((idx, digit));
                            }
                        }
                    }
                    if !eliminated.is_empty() && state.eliminate(&eliminated) {
                        return Some(SolveStep {
                            technique: Technique::NakedPair,
                            cell: None,
                            digit: None,
                            eliminated,
                        });
                    }
                }
            }
        }
    }
    None
}

fn apply_hidden_pair(state: &mut CandidateState) -> Option<SolveStep> {
    for unit in units() {
        for d1 in 1..=8 {
            for d2 in d1 + 1..=9 {
                let pair_mask = crate::solver::candidates::digit_bit(d1)
                    | crate::solver::candidates::digit_bit(d2);
                let mut locations = Vec::new();
                for &idx in &unit {
                    if state.board.get_index(idx) == 0
                        && state.masks[idx] & pair_mask != 0
                        && (mask_contains(state.masks[idx], d1)
                            || mask_contains(state.masks[idx], d2))
                    {
                        locations.push(idx);
                    }
                }
                let d1_locations = locations
                    .iter()
                    .filter(|&&idx| mask_contains(state.masks[idx], d1))
                    .copied()
                    .collect::<Vec<_>>();
                let d2_locations = locations
                    .iter()
                    .filter(|&&idx| mask_contains(state.masks[idx], d2))
                    .copied()
                    .collect::<Vec<_>>();
                if d1_locations.len() == 2 && d1_locations == d2_locations {
                    let mut eliminated = Vec::new();
                    for idx in d1_locations {
                        let extra = state.masks[idx] & !pair_mask & ALL_DIGITS_MASK;
                        for digit in 1..=9 {
                            if mask_contains(extra, digit) {
                                eliminated.push((idx, digit));
                            }
                        }
                    }
                    if !eliminated.is_empty() && state.eliminate(&eliminated) {
                        return Some(SolveStep {
                            technique: Technique::HiddenPair,
                            cell: None,
                            digit: None,
                            eliminated,
                        });
                    }
                }
            }
        }
    }
    None
}

fn apply_naked_triple(state: &mut CandidateState) -> Option<SolveStep> {
    for unit in units() {
        let empty = unit
            .iter()
            .copied()
            .filter(|&idx| {
                state.board.get_index(idx) == 0 && (2..=3).contains(&mask_count(state.masks[idx]))
            })
            .collect::<Vec<_>>();
        for a in 0..empty.len() {
            for b in a + 1..empty.len() {
                for c in b + 1..empty.len() {
                    let indexes = [empty[a], empty[b], empty[c]];
                    let union = indexes
                        .iter()
                        .fold(0u16, |acc, &idx| acc | state.masks[idx]);
                    if mask_count(union) != 3 {
                        continue;
                    }
                    let mut eliminated = Vec::new();
                    for &idx in &unit {
                        if indexes.contains(&idx) || state.board.get_index(idx) != 0 {
                            continue;
                        }
                        for digit in 1..=9 {
                            if mask_contains(union, digit) && mask_contains(state.masks[idx], digit)
                            {
                                eliminated.push((idx, digit));
                            }
                        }
                    }
                    if !eliminated.is_empty() && state.eliminate(&eliminated) {
                        return Some(SolveStep {
                            technique: Technique::NakedTriple,
                            cell: None,
                            digit: None,
                            eliminated,
                        });
                    }
                }
            }
        }
    }
    None
}

fn apply_hidden_triple(state: &mut CandidateState) -> Option<SolveStep> {
    for unit in units() {
        for d1 in 1..=7 {
            for d2 in d1 + 1..=8 {
                for d3 in d2 + 1..=9 {
                    let digits = [d1, d2, d3];
                    let triple_mask = digits.iter().fold(0u16, |acc, &digit| {
                        acc | crate::solver::candidates::digit_bit(digit)
                    });
                    let mut locations = Vec::new();
                    let mut each_digit_present = true;
                    for &digit in &digits {
                        let digit_locations = unit
                            .iter()
                            .copied()
                            .filter(|&idx| {
                                state.board.get_index(idx) == 0
                                    && mask_contains(state.masks[idx], digit)
                            })
                            .collect::<Vec<_>>();
                        if digit_locations.is_empty() || digit_locations.len() > 3 {
                            each_digit_present = false;
                            break;
                        }
                        for idx in digit_locations {
                            if !locations.contains(&idx) {
                                locations.push(idx);
                            }
                        }
                    }
                    if !each_digit_present || locations.len() != 3 {
                        continue;
                    }
                    let mut eliminated = Vec::new();
                    for idx in locations {
                        let extra = state.masks[idx] & !triple_mask & ALL_DIGITS_MASK;
                        for digit in 1..=9 {
                            if mask_contains(extra, digit) {
                                eliminated.push((idx, digit));
                            }
                        }
                    }
                    if !eliminated.is_empty() && state.eliminate(&eliminated) {
                        return Some(SolveStep {
                            technique: Technique::HiddenTriple,
                            cell: None,
                            digit: None,
                            eliminated,
                        });
                    }
                }
            }
        }
    }
    None
}

fn ordered_used_techniques(steps: &[SolveStep]) -> Vec<Technique> {
    let order = [
        Technique::NakedSingle,
        Technique::HiddenSingle,
        Technique::LockedCandidate,
        Technique::NakedPair,
        Technique::HiddenPair,
        Technique::NakedTriple,
        Technique::HiddenTriple,
        Technique::XWing,
    ];
    order
        .iter()
        .copied()
        .filter(|technique| steps.iter().any(|step| step.technique == *technique))
        .collect()
}

pub fn technique_score(technique: Technique) -> u32 {
    match technique {
        Technique::NakedSingle => 10,
        Technique::HiddenSingle => 20,
        Technique::LockedCandidate => 35,
        Technique::NakedPair => 50,
        Technique::HiddenPair => 60,
        Technique::NakedTriple => 80,
        Technique::HiddenTriple => 90,
        Technique::XWing => 120,
    }
}

fn classify_human_result(
    clue_count: usize,
    solved: bool,
    used: &[Technique],
    _rating_score: u32,
) -> Difficulty {
    let has_medium = used.iter().any(|technique| {
        matches!(
            technique,
            Technique::LockedCandidate | Technique::NakedPair | Technique::HiddenPair
        )
    });
    let has_hard = used.iter().any(|technique| {
        matches!(
            technique,
            Technique::NakedTriple | Technique::HiddenTriple | Technique::XWing
        )
    });

    if clue_count <= 21 {
        return Difficulty::Extreme;
    }
    if !solved {
        return Difficulty::Expert;
    }
    if has_hard {
        return Difficulty::Hard;
    }
    if has_medium {
        return Difficulty::Medium;
    }
    Difficulty::Easy
}
