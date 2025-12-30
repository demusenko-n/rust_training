use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    path: PathBuf,

    #[arg(long, short)]
    output_file: Option<PathBuf>,

    #[arg(long, default_value_t = 1)]
    max_depth: u32,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let result = day4::v1::index_directory_async(&args.path, args.max_depth).await?;
    let json = serde_json::to_string_pretty(&result.map)?;

    if let Some(output_path) = args.output_file {
        tokio::fs::write(&output_path, json).await?;
    } else {
        println!("{json}");
    };

    Ok(())
}
