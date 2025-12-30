use std::{fs, path::PathBuf};

use anyhow;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    path: PathBuf,

    #[arg(long, short)]
    output_file: Option<PathBuf>,

    #[arg(long, default_value_t = 1)]
    max_depth: u32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let result = day2::index_directory(&args.path, args.max_depth)?;

    if !result.errors.errors.is_empty() {
        println!("{:#?}", result.errors);
    }

    let result = format!("{:#?}", result.map);
    if let Some(output_path) = args.output_file {
        fs::write(&output_path, result)?;
    } else {
        println!("{result}");
    };

    Ok(())
}
