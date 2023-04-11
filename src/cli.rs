use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the checks configuration file, which is demanded.
    #[arg(short, long = "rule-file", value_parser = check_file_existence)]
    rule_file: PathBuf,

    /// The path to the source code file that will be checked.
    #[arg(short, long = "src-file", value_parser = check_file_existence)]
    src_file: PathBuf,

    /// The path to the output file.
    #[arg(short, long, value_parser = check_dir_existence)]
    output: PathBuf,
}

fn check_file_existence(p: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(p);
    if p.exists() {
        Ok(p)
    } else {
        Err(format!("The file you provide doesn't exist: {p:?}"))
    }
}

fn check_dir_existence(p: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(p);
    let parent = match p.parent() {
        Some(v) => v,
        None => return Err(format!("The file path you provid is invalid: {p:?}")),
    };

    if parent.exists() {
        Ok(p)
    } else {
        Err(format!(
            "The file's directory you provide doesn't exist: {p:?}"
        ))
    }
}
