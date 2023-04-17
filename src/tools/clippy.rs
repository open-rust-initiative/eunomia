use super::{Checker, Command, FilteredOutput};
use crate::{utils, Result};
use std::process::Output;

pub struct ClippyOpt<'c> {
    cmd: Command<'c>,
}

impl<'c> ClippyOpt<'c> {
    pub fn from_command(cmd: Command<'c>) -> Self {
        Self { cmd }
    }
}

impl Checker for ClippyOpt<'_> {
    fn check(&self) -> Result<Output> {
        utils::execute_for_output(self.cmd.app, self.cmd.args, self.cmd.envs.to_vec())
    }
    fn filter_output(&self, output: &Output) -> FilteredOutput {
        // Clippy output is usually stderr type, so we keep stdout empty.
        let stdout = Vec::new();

        let mut stderr = Vec::new();

        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

        // Clippy splits each output by double newline characters,
        let mut stderr_iter = stderr_str.split("\n\n").peekable();
        // The first line of Clippy out is always `Checking <crate name> ...`
        // which is irrelevent in this case, so we will remove the first line of the first item.
        if let Some(first) = stderr_iter
            .next()
            .map(|s| s.lines().skip(1).collect::<Vec<_>>())
        {
            stderr.push(first.join("\n"));
        }

        while let Some(item) = stderr_iter.next() {
            // Skip the last section of Clippy output because is usually a summary message such as
            // 'warning: `<crate name>` generated x errors....'
            if stderr_iter.peek().is_some() {
                stderr.push(item.to_string())
            }
        }

        FilteredOutput { stdout, stderr }
    }
}
