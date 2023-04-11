use clap::Parser;
use eunomia::cli::Args;

fn main() {
    let args = Args::parse();

    dbg!(args);
}
