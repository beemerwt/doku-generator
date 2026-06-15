use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SymmetryMode {
    None,
    Rotational180,
}

impl fmt::Display for SymmetryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymmetryMode::None => f.write_str("none"),
            SymmetryMode::Rotational180 => f.write_str("rotational180"),
        }
    }
}
