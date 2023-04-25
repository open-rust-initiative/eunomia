use clap::Parser;
use eunomia::cli::Args;
use eunomia::Result;

fn main() -> Result<()> {
    let args = Args::parse();
    args.run()?;
    dbg!(args);

    Ok(())
}
