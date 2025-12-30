use clap::Parser;
use std::{fs::OpenOptions, num::NonZero, path::PathBuf};
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    path: PathBuf,

    #[arg(long, short)]
    output_file: Option<PathBuf>,

    #[arg(long, default_value_t = 1)]
    max_depth: u32,

    #[arg(
        long,
        help = "Maximum number of worker threads [default: all available threads]"
    )]
    max_threads: Option<NonZero<usize>>,
}

fn main() -> anyhow::Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("logs/day3.log")?;

    let file_appender = tracing_appender::rolling::never("logs", "day3.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_writer(file_writer))
        .init();

    let args = Args::parse();
    let max_threads = match args.max_threads {
        Some(v) => v,
        None => std::thread::available_parallelism()?,
    };

    println!("{args:?} {max_threads}");

    // let resultv1 = day3::v1::index_directory_thr(&args.path, args.max_depth, max_threads)?;
    // println!("{resultv1:#?}");

    let resultv2 = day3::v2::index_directory_thr(&args.path, args.max_depth, max_threads)?;
    let json = serde_json::to_string_pretty(&resultv2.map)?;

    if let Some(output_path) = args.output_file {
        std::fs::write(&output_path, json)?;
    } else {
        println!("{json}");
    };

    // assert!(resultv1.map == resultv2.map);

    Ok(())
}
