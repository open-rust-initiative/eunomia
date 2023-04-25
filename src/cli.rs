use std::path::PathBuf;

use clap::Parser;

use crate::parser::{JsonStruct, RulesCfg};
use crate::tools::miri::MiriOpt;
use crate::tools::LintsOpt;
use crate::{utils, Error, Result};

pub enum ToolCmd {
    Clippy(LintsOpt),
    Rustc(LintsOpt),
    Miri(MiriOpt),
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the checks configuration file, which is demanded.
    #[arg(short, long = "rule-file", value_parser = check_file_existence)]
    pub rule_file: PathBuf,

    /// Overrides the path to the source code file that will be checked.
    #[arg(short, long = "src-file", value_parser = check_file_existence)]
    pub src_file: Option<PathBuf>,

    /// The path to the output file.
    #[arg(short, long, value_parser = check_dir_existence)]
    pub output: Option<PathBuf>,
}

impl Args {
    // TODO: deserialize a guidelines json file, generate a map with guideline id as
    // keys, then read ids from RulesCfg, then for each id, find out which tools to
    // use via the map, and define running options with src_path for each tool, then run execute
    // with those options one by one, and finally, after getting the output file, write it
    // as the desired output path.
    pub fn run(&self) -> Result<()> {
        let rule_content = utils::read_to_string(&self.rule_file)?;
        let rule_cfg = RulesCfg::deserialize(&rule_content)?;

        let _src_path = if let Some(path) = &self.src_file {
            // TODO: use log crate's `info!`.
            println!("overriding src path from commandline");
            path.as_path()
        } else {
            rule_cfg.file_path
        };
        let _output_path = if let Some(path) = &self.output {
            path.clone()
        } else {
            PathBuf::from("output.json")
        };

        Ok(())
    }
}

fn check_file_existence(p: &str) -> Result<PathBuf> {
    let p = PathBuf::from(p);
    (p.exists())
        .then_some(p.clone())
        .ok_or(Error::PathNotExist("file", p).into())
}

fn check_dir_existence(p: &str) -> Result<PathBuf> {
    let p = PathBuf::from(p);
    let parent = p.parent().ok_or(Error::InvalidFilePath(p.clone()))?;
    (parent.exists())
        .then_some(p.clone())
        .ok_or(Error::OrphanFilePath(p).into())
}
