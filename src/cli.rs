use std::collections::{HashMap, HashSet};
use std::io::stdin;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::Parser;

use crate::parser::{
    CheckInfo, CodingGuidelines, Guideline, GuidelineID, JsonStruct, Output, RulesCfg,
};
use crate::tools::{Checker, LintsOpt, SupportedTool};
use crate::{utils, Error, Result};

const GUILDELINES_CONTENT: &str = include_str!("guidelines.json");

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
    // TODO: read ids from RulesCfg, then for each id, find out which tools to
    // use via the map, and define running options with src_path for each tool, then run execute
    // with those options one by one, and finally, after getting the output file, write it
    // as the desired output path.
    pub fn run(&self) -> Result<()> {
        let guidelines = CodingGuidelines::deserialize(GUILDELINES_CONTENT)?;
        let gl_map = guidelines.to_hashmap();

        let rule_content = utils::read_to_string(&self.rule_file)?;
        let rule_cfg = RulesCfg::deserialize(&rule_content)?;

        let src_path = if let Some(path) = &self.src_file {
            // TODO: use log crate's `info!`.
            println!("overriding src path from commandline");
            path.as_path()
        } else {
            rule_cfg.file_path
        };
        let output_path = if let Some(path) = &self.output {
            path.clone()
        } else {
            PathBuf::from("output.json")
        };

        let check_info_vec = run_checks(src_path, &rule_cfg.coding_guidelines, &gl_map)?;
        let output = Output::from(check_info_vec).to_json_string_pretty()?;
        utils::write_to_file(output, output_path)?;
        Ok(())
    }
}

fn run_checks(
    path: &Path,
    ids: &HashSet<GuidelineID>,
    gl_map: &HashMap<&GuidelineID, &Guideline>,
) -> Result<Vec<CheckInfo>> {
    let mut result = vec![];
    let has_cargo = utils::command_exist("cargo", &["-V"])?;

    if !has_cargo {
        // TODO: use log `info!`; use proper interactive method, which allows user to
        // pass `-y` in order to skip interaction
        print!(
            "We couldn't find `cargo`'s executable to run, make sure it's \
        in the path. Because some tools (such as miri) could not run without it, the result \
        might be incomplete, continue? [y/N]: "
        );
        let mut choice = String::new();
        stdin().read_line(&mut choice)?;

        // I just what to print a new line, is that hard?
        #[allow(clippy::println_empty_string)]
        match choice.trim() {
            "n" | "N" => exit(0),
            _ => println!(""),
        }
    }

    // FIXME: refractor below code.
    // as it's basically copy-pastes and make it harder to add new tool support.
    let mut maybe_clippy_opt = None;
    let mut maybe_rustc_opt = None;

    for id in ids {
        let Some(Guideline { tool, .. }) = gl_map.get(id) else { continue; };
        for t in tool {
            match t.name {
                SupportedTool::Clippy => {
                    if maybe_clippy_opt.is_none() {
                        maybe_clippy_opt = Some(LintsOpt {
                            use_cargo: has_cargo,
                            is_clippy: true,
                            path: path.to_path_buf(),
                            ..Default::default()
                        });
                    }
                    maybe_clippy_opt
                        .as_mut()
                        .expect("lint option should be initialized at this point")
                        .lint_names
                        .push(t.ident.to_string());
                }
                SupportedTool::Rustc => {
                    if maybe_rustc_opt.is_none() {
                        maybe_rustc_opt = Some(LintsOpt {
                            use_cargo: has_cargo,
                            path: path.to_path_buf(),
                            ..Default::default()
                        });
                    }
                    maybe_rustc_opt
                        .as_mut()
                        .expect("lint option should be initialized at this point")
                        .lint_names
                        .push(t.ident.to_string());
                }
                // FIXME
                _ => unimplemented!("support for '{}' is not implemented.", t.name),
            }
        }
    }

    if let Some(opt) = maybe_clippy_opt {
        let output = opt.check()?;
        let filtered = opt.filter_output(&output);
        for err in filtered.stderr {
            result.push(opt.check_info(&err)?);
        }
    }
    if let Some(opt) = maybe_rustc_opt {
        let output = opt.check()?;
        let filtered = opt.filter_output(&output);
        for err in filtered.stderr {
            result.push(opt.check_info(&err)?);
        }
    }

    Ok(result)
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
