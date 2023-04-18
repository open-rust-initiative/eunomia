//! Lints implementation of the [`Checker`] trait,
//! works for rustc lints and clippy lints.

use regex::Regex;

use super::{Checker, Command, FilteredOutput, SupportedTool};
use crate::parser::CheckInfo;
use crate::{utils, Result};
use std::path::PathBuf;
use std::process::Output;

pub struct LintsOpt<'c> {
    pub is_clippy: bool,
    pub cmd: Command<'c>,
}

impl Checker for LintsOpt<'_> {
    fn check(&self) -> Result<Output> {
        utils::execute_for_output(self.cmd.app, self.cmd.args, self.cmd.envs.to_vec())
    }
    fn filter_output(&self, output: &Output) -> FilteredOutput {
        // rustc_lint result is usually stderr type, so we keep stdout empty.
        let stdout = Vec::new();

        let mut stderr = Vec::new();
        let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();

        // rustc_lint divide each output by double newline characters,
        let mut stderr_iter = stderr_str.split("\n\n").peekable();
        if let Some(first) = stderr_iter.next().map(skip_first_line_when_possible) {
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

    /// Generalize clippy output to [`CheckInfo`] struct.
    ///
    /// Clippy output is contructed using similar fashion,
    /// where the first line contains help information summary,
    /// the line followed by '-->' contains source file location, start line and column number,
    /// and following is the code, etc.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let raw_result: "error: range is out of bounds
    ///  --> src/clippy.rs:12:19
    ///   |
    ///12 |     let _ = &x[2..9];
    ///   |                   ^
    ///   |
    ///   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing
    ///   = note: `#[deny(clippy::out_of_bounds_indexing)]` on by default"
    ///
    /// let info = ClippyOpt::check_info(raw_result).unwrap();
    /// assert_eq!(info.help_info, Some("range is out of bounds".to_string()));
    /// assert_eq!(info.defect_name, "out_of_bounds_indexing".to_string());
    /// ```
    fn check_info(&self, raw_result: &str) -> Result<CheckInfo> {
        let mut lines = raw_result.trim().lines();

        // First line contains help message
        let help_info = lines
            .next()
            .and_then(|s| s.split_once(':'))
            .map_or(String::new(), |(_, msg)| msg.trim().to_string());

        // Second line might contains position info
        let (file_path, begin_line, column) = match lines.next().map(|s| {
            s.trim()
                .trim_start_matches("--> ")
                .split(':')
                .collect::<Vec<_>>()
        }) {
            Some(v) if v.len() == 3 => (
                Some(PathBuf::from(v[0])),
                v[1].trim().parse::<usize>().ok(),
                v[2].trim().parse::<usize>().ok(),
            ),
            _ => (None, None, None),
        };

        let mut code_string = String::new();
        let mut extracted_info = ExtractedCheckInfo::default();
        let help_lint_name_regex = if self.is_clippy {
            Some(Regex::new(r"rust-clippy/master/index.html#(\w+)")?)
        } else {
            None
        };
        let note_lint_name_regex = Some(Regex::new(r"(`-(?:W|D) (.*?)`)|(\((.*?)\)\])")?);
        // A flag to mark whether the program is looking for error code snippets or not,
        // otherwise it's possible that a suggestion snippet will be taken by mistake.
        let mut looking_for_code = true;

        for line in lines {
            let trimmed = line.trim();
            // FIXME: this method is dumb, it's expensive and could cause false positive if
            // a string variable has '= help: ' inside of it.
            if trimmed.contains("= help: ") {
                looking_for_code = false;
                update_info_from_help_or_note(
                    &mut extracted_info,
                    trimmed,
                    help_lint_name_regex.as_ref(),
                );
            } else if trimmed.contains("= note: ") {
                looking_for_code = false;
                update_info_from_help_or_note(
                    &mut extracted_info,
                    trimmed,
                    note_lint_name_regex.as_ref(),
                );
            } else if looking_for_code {
                if let Some(code) = maybe_code_line(trimmed) {
                    code_string.push_str(code);
                }
            }
        }

        Ok(CheckInfo {
            file_path,
            defect_name: extracted_info.defect_name,
            tool: if self.is_clippy {
                SupportedTool::Clippy
            } else {
                SupportedTool::Rustc
            },
            begin_line,
            column,
            code_string,
            help_info,
            additional_help_info: extracted_info.additional_help,
            // FIXME: currently we haven't figured out a way to get the end line.
            // TODO: get guideline relation map, and put them here.
            ..Default::default()
        })
    }
}

#[derive(Default)]
/// Subset of [`CheckInfo`], that could be extracted from 'help' and 'note' section
/// of clippy output.
struct ExtractedCheckInfo {
    defect_name: String,
    additional_help: String,
}

/// The first line of lints result is inconsistent depending on different scenario,
/// it could be the result straight away such as `warning: xxx` or `error: xxx`.
/// Or, it could be notification such as `    Checking crate xxx` or `    Blocking waiting for ...`
/// in that case we can skip that those information.
fn skip_first_line_when_possible(s: &str) -> Vec<&str> {
    let mut result = vec![];
    let mut lines = s.lines();
    for line in lines.by_ref() {
        if !line.trim_start().starts_with(['w', 'e']) {
            continue;
        } else {
            result.push(line);
            break;
        }
    }
    result.extend(lines);
    result
}

fn update_info_from_help_or_note(info: &mut ExtractedCheckInfo, line: &str, re: Option<&Regex>) {
    info.additional_help
        .push_str(&format!("{}\n", line.trim_start_matches("= ")));
    if let Some(re) = re {
        if let Some(lint_name) = utils::regex_utils::capture_last(re, line) {
            info.defect_name = lint_name.to_string();
        }
    }
}

fn maybe_code_line(s: &str) -> Option<&str> {
    if let Some((num, code)) = s.split_once(" | ") {
        if num.trim().parse::<usize>().is_ok() {
            return Some(code);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::utils::regex_utils;
    use super::{maybe_code_line, Regex};

    #[test]
    fn test_extract_lint_name_from_note() {
        let note_str = "= note: `#[warn(clippy::bool_comparison)]` on by default";
        let note_str_1 = "= note: requested on the command line with `-W non-ascii-idents`";

        let note_regex = Regex::new(r"(`-(?:W|D) (.*?)`)|(\((.*?)\)\])").unwrap();

        let name = regex_utils::capture_last(&note_regex, note_str);
        let name_1 = regex_utils::capture_last(&note_regex, note_str_1);

        assert_eq!(name, Some("clippy::bool_comparison"));
        assert_eq!(name_1, Some("non-ascii-idents"));
    }

    #[test]
    fn test_extract_lint_name_from_help() {
        let help_regex = Regex::new(r"rust-clippy/master/index.html#(?P<name>(\w+))").unwrap();
        let help_str = "= help: for further information visit \
        https://rust-lang.github.io/rust-clippy/master/index.html#bool_comparison";

        let name = regex_utils::get_named_match("name", &help_regex, help_str);
        assert_eq!(name, Some("bool_comparison".to_string()));
    }

    #[test]
    fn test_extract_code() {
        let code_line_1 = "18 |     if flag == true {}";
        let code_line_2 = "   |        ^^^^^^^^^^^^ help: try simplifying it as shown: `flag`";
        let code_line_3 = "   |     ";

        assert_eq!(maybe_code_line(code_line_1), Some("    if flag == true {}"));
        assert_eq!(maybe_code_line(code_line_2), None);
        assert_eq!(maybe_code_line(code_line_3), None);
    }
}
