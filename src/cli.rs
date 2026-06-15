use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    db::{close_database, init_db, insert_puzzles, open_database},
    difficulty::Difficulty,
    generator::symmetry::SymmetryMode,
    generator::{generate_one_cancellable, GeneratedPuzzle, GenerationStats},
    GENERATOR_VERSION,
};

#[derive(Debug, Parser)]
#[command(name = "doku-generator")]
#[command(about = "Generate, rate, and store Sudoku puzzles for Doku")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Generate(GenerateArgs),
}

#[derive(Debug, Args)]
struct GenerateArgs {
    #[arg(long)]
    database: String,

    #[arg(long, value_enum)]
    difficulty: Difficulty,

    #[arg(long)]
    count: usize,

    #[arg(long)]
    seed: Option<u64>,

    #[arg(long)]
    workers: Option<usize>,

    #[arg(long, value_enum, default_value_t = SymmetryMode::None)]
    symmetry: SymmetryMode,

    #[arg(long, default_value_t = 1000)]
    batch_size: usize,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate(args) => generate(args),
    }
}

fn generate(args: GenerateArgs) -> Result<()> {
    if args.count == 0 {
        bail!("--count must be greater than zero");
    }
    if args.batch_size == 0 {
        bail!("--batch-size must be greater than zero");
    }

    let default_workers = num_cpus::get().saturating_sub(1).max(1);
    let workers = args.workers.unwrap_or(default_workers).max(1);
    let seed = args.seed.unwrap_or(0);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let shutdown = install_shutdown_handler()?;
    let mut conn = open_database(&args.database)?;
    init_db(&conn)?;

    println!("Database: {}", args.database);
    println!("Difficulty: {}", args.difficulty);
    println!("Target inserts: {}", args.count);
    println!("Symmetry: {}", args.symmetry);
    println!("Workers: {}", if workers == 1 { 1 } else { workers });
    println!("Batch size: {}", args.batch_size);
    if workers != 1 {
        println!("Parallel generation is not enabled yet; using deterministic single-threaded generation.");
    }
    println!();

    let started = Instant::now();
    let mut stats = GenerationStats::default();
    let mut inserted = 0usize;
    let mut duplicates = 0usize;
    let mut batch: Vec<GeneratedPuzzle> = Vec::with_capacity(args.batch_size);

    while inserted < args.count && !shutdown.load(Ordering::SeqCst) {
        batch.clear();
        while batch.len() < args.batch_size
            && inserted + batch.len() < args.count
            && !shutdown.load(Ordering::SeqCst)
        {
            let generated = generate_one_cancellable(
                args.difficulty,
                &mut rng,
                args.symmetry,
                &mut stats,
                || shutdown.load(Ordering::SeqCst),
            )?;
            let Some(generated) = generated else {
                break;
            };
            batch.push(generated);
        }

        if !batch.is_empty() {
            let actually_inserted = insert_puzzles(&mut conn, &batch)?;
            inserted += actually_inserted;
            duplicates += batch.len().saturating_sub(actually_inserted);
            println!("Inserted {inserted}/{} puzzles", args.count);
        }
    }

    let interrupted = shutdown.load(Ordering::SeqCst);
    let elapsed = started.elapsed();
    let seconds = elapsed.as_secs_f64();
    let per_second = if seconds > 0.0 {
        inserted as f64 / seconds
    } else {
        inserted as f64
    };

    println!();
    close_database(conn)?;

    if interrupted {
        println!("Interrupted. SQLite connection closed.");
    } else {
        println!("Done.");
    }
    println!();
    println!("Inserted: {inserted}");
    println!("Duplicates ignored: {duplicates}");
    println!("Candidates attempted: {}", stats.candidates_attempted);
    println!("Accepted by uniqueness: {}", stats.accepted_by_uniqueness);
    println!("Accepted by difficulty: {}", stats.accepted_by_difficulty);
    println!("Elapsed: {:.1}s", seconds);
    println!("Inserted puzzles/sec: {:.1}", per_second);
    println!("Generator version: {GENERATOR_VERSION}");

    Ok(())
}

fn install_shutdown_handler() -> Result<Arc<AtomicBool>> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&shutdown);
    ctrlc::set_handler(move || {
        if !handler_flag.swap(true, Ordering::SeqCst) {
            eprintln!(
                "Ctrl+C received. Finishing current puzzle/batch, closing SQLite, then exiting..."
            );
        }
    })?;
    Ok(shutdown)
}
