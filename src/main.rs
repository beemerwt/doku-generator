mod board;
mod cli;
mod db;
mod difficulty;
mod generator;
mod hash;
mod rating;
mod solver;

use anyhow::Result;

pub const GENERATOR_VERSION: u32 = 1;

fn main() -> Result<()> {
    cli::run()
}
