use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::{prelude::*, ThreadPool, ThreadPoolBuilder};

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
    let thread_pool = build_thread_pool(workers)?;
    let mut conn = open_database(&args.database)?;
    init_db(&conn)?;

    println!("Database: {}", args.database);
    println!("Difficulty: {}", args.difficulty);
    println!("Target inserts: {}", args.count);
    println!("Symmetry: {}", args.symmetry);
    println!("Workers: {}", if workers == 1 { 1 } else { workers });
    println!("Batch size: {}", args.batch_size);
    println!(
        "Determinism: guaranteed for fixed seed, difficulty, symmetry, batch size, and worker count"
    );
    println!();

    let started = Instant::now();
    let mut stats = GenerationStats::default();
    let mut inserted = 0usize;
    let mut duplicates = 0usize;
    let mut db_insert_nanos = 0u128;
    let mut batch: Vec<GeneratedPuzzle> = Vec::with_capacity(args.batch_size);

    while inserted < args.count && !shutdown.load(Ordering::SeqCst) {
        batch.clear();
        let requested = args.batch_size.min(args.count - inserted);
        batch = generate_batch(
            requested,
            args.difficulty,
            args.symmetry,
            &mut rng,
            &mut stats,
            &shutdown,
            thread_pool.as_ref(),
        )?;

        if !batch.is_empty() {
            let started = Instant::now();
            let actually_inserted = insert_puzzles(&mut conn, &batch)?;
            db_insert_nanos += started.elapsed().as_nanos();
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
    println!(
        "Time solved board generation: {:.3}s",
        nanos_to_seconds(stats.solved_generation_nanos)
    );
    println!(
        "Time digging: {:.3}s",
        nanos_to_seconds(stats.digging_nanos)
    );
    println!(
        "Time uniqueness checks: {:.3}s",
        nanos_to_seconds(stats.uniqueness_nanos)
    );
    println!("Time rating: {:.3}s", nanos_to_seconds(stats.rating_nanos));
    println!(
        "Time database insertion: {:.3}s",
        nanos_to_seconds(db_insert_nanos)
    );
    println!("Generator version: {GENERATOR_VERSION}");

    Ok(())
}

fn build_thread_pool(workers: usize) -> Result<Option<ThreadPool>> {
    if workers <= 1 {
        return Ok(None);
    }
    Ok(Some(ThreadPoolBuilder::new().num_threads(workers).build()?))
}

fn generate_batch(
    requested: usize,
    difficulty: Difficulty,
    symmetry: SymmetryMode,
    rng: &mut ChaCha8Rng,
    stats: &mut GenerationStats,
    shutdown: &Arc<AtomicBool>,
    thread_pool: Option<&ThreadPool>,
) -> Result<Vec<GeneratedPuzzle>> {
    if requested == 0 || shutdown.load(Ordering::SeqCst) {
        return Ok(Vec::new());
    }

    let job_seeds = (0..requested).map(|_| rng.gen::<u64>()).collect::<Vec<_>>();

    if let Some(thread_pool) = thread_pool {
        let generated = thread_pool.install(|| {
            job_seeds
                .par_iter()
                .map(|&job_seed| {
                    let mut local_rng = ChaCha8Rng::seed_from_u64(job_seed);
                    let mut local_stats = GenerationStats::default();
                    let generated = generate_one_cancellable(
                        difficulty,
                        &mut local_rng,
                        symmetry,
                        &mut local_stats,
                        || shutdown.load(Ordering::SeqCst),
                    )?;
                    Ok::<_, anyhow::Error>((generated, local_stats))
                })
                .collect::<Vec<_>>()
        });

        let mut batch = Vec::with_capacity(requested);
        for item in generated {
            let (generated, local_stats) = item?;
            stats.merge(&local_stats);
            if let Some(generated) = generated {
                batch.push(generated);
            }
        }
        Ok(batch)
    } else {
        let mut batch = Vec::with_capacity(requested);
        for job_seed in job_seeds {
            if shutdown.load(Ordering::SeqCst) {
                break;
            }
            let mut local_rng = ChaCha8Rng::seed_from_u64(job_seed);
            let mut local_stats = GenerationStats::default();
            let generated = generate_one_cancellable(
                difficulty,
                &mut local_rng,
                symmetry,
                &mut local_stats,
                || shutdown.load(Ordering::SeqCst),
            )?;
            stats.merge(&local_stats);
            let Some(generated) = generated else {
                break;
            };
            batch.push(generated);
        }
        Ok(batch)
    }
}

fn nanos_to_seconds(nanos: u128) -> f64 {
    nanos as f64 / 1_000_000_000.0
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
