//! This module contains definition/implementation for tools that this program
//! supports, such as `clippy`, `miri`, etc...
//!
//! All relavent definition should be declared here, just in case more tools will be
//! added or deleted in the future.

mod lints;

pub use self::lints::LintsOpt;

use crate::{parser::CheckInfo, Result};
use std::{fmt::Display, process::Output, str::FromStr};

/// Simple struct contains the name of executable, its args, and env vars in order
/// to form a full runable command.
#[derive(Default)]
pub struct Command<'c> {
    pub app: &'c str,
    pub args: &'c [&'c str],
    pub envs: &'c [(&'c str, &'c str)],
}

impl Display for Command<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self.args.join(" ");
        let envs = self
            .envs
            .iter()
            .map(|(k, v)| format!("{k}=\"{v}\""))
            .collect::<Vec<String>>()
            .join(" ");
        f.write_fmt(format_args!("{envs} {} {}", self.app, args))
    }
}

pub trait Checker {
    /// Get output by running commands.
    fn check(&self) -> Result<Output>;
    /// Extract only the useful information from output while splitting
    /// them into different sections of multiple checking results.
    ///
    /// Note that the outputs are lossy `String` types, which is suitable
    /// for printing. If you want a generalized output types,
    /// implement [`Checker::check_info`] to interpret each output message
    /// into a generalized [`CheckInfo`] type.
    fn filter_output(&self, output: &Output) -> FilteredOutput;
    /// Generalize a string of output message to [`CheckInfo`] struct.
    fn check_info(&self, raw_result: &str) -> Result<CheckInfo>;
}

/// The output of a tool could have multiple sections of checked result,
/// so we need to split them, and extract only useful information.
///
/// For example, clippy's output usually looks like:
///
/// ```text
/// warning: xxx
///   --> xxxxx
///    ...
///    = help: ...
///
/// error: xxx
///   ...
///    = help: ...
///
/// ```
///
/// Therefore we want to extract all the sections from `warning` or `error`
/// to the line of `help: ...` in this example
#[derive(Debug)]
pub struct FilteredOutput {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SupportedTool {
    Clippy,
    #[default]
    Rustc,
    Miri,
    Sanitizer,
}

impl SupportedTool {
    // TODO: put this method in a derive macro
    pub fn all() -> Vec<Self> {
        vec![Self::Clippy, Self::Rustc, Self::Miri, Self::Sanitizer]
    }
}

// TODO: put this method in a derive macro
impl Display for SupportedTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SupportedTool::*;
        let str = match self {
            Clippy => "clippy",
            Rustc => "rustc",
            Miri => "miri",
            Sanitizer => "sanitizer",
        };
        f.write_str(str)
    }
}

// TODO: put this method in a derive macro
impl FromStr for SupportedTool {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clippy" => Ok(Self::Clippy),
            "rustc" => Ok(Self::Rustc),
            "miri" => Ok(Self::Miri),
            "sanitizer" => Ok(Self::Sanitizer),
            _ => Err(crate::Error::ParseUnsupportedEnumVariant(
                "tool name",
                s.to_string(),
                SupportedTool::all()
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            )),
        }
    }
}

#[cfg(test)]
mod common_tool_tests {
    use super::Command;

    #[test]
    fn command_lifetime_and_to_string() {
        let cmd = Command {
            app: "cargo",
            args: &["clippy", "--fix", "--", "-W", "clippy::all"],
            envs: &[("TESTNAME", "needless_match")],
        };
        assert_eq!(
            cmd.to_string(),
            "TESTNAME=\"needless_match\" cargo clippy --fix -- -W clippy::all"
        );
        let app = "cargo";
        let args = vec!["add", "serde", "--features", "derive"];
        let envs = [("MY_ENV_VAR", "example")];
        let cmd = Command {
            app,
            args: &args,
            envs: &envs,
        };
        assert_eq!(
            cmd.to_string(),
            "MY_ENV_VAR=\"example\" cargo add serde --features derive"
        );
    }
}
