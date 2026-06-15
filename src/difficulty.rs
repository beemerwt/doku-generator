use anyhow::{anyhow, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
    Extreme,
    Mixed,
}

impl Difficulty {
    pub fn clue_range(self) -> Option<(usize, usize)> {
        match self {
            Difficulty::Easy => Some((38, 45)),
            Difficulty::Medium => Some((32, 37)),
            Difficulty::Hard => Some((26, 31)),
            Difficulty::Expert => Some((22, 27)),
            Difficulty::Extreme => Some((17, 21)),
            Difficulty::Mixed => None,
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
            Difficulty::Expert => "expert",
            Difficulty::Extreme => "extreme",
            Difficulty::Mixed => "mixed",
        };
        f.write_str(value)
    }
}

impl FromStr for Difficulty {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_ascii_lowercase().as_str() {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            "expert" => Ok(Difficulty::Expert),
            "extreme" => Ok(Difficulty::Extreme),
            "mixed" => Ok(Difficulty::Mixed),
            _ => Err(anyhow!("unknown difficulty: {input}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_difficulty() {
        assert_eq!("easy".parse::<Difficulty>().unwrap(), Difficulty::Easy);
        assert_eq!("mixed".parse::<Difficulty>().unwrap(), Difficulty::Mixed);
        assert!("impossible".parse::<Difficulty>().is_err());
    }
}
