use regex::Regex;

use super::{Checker, Command, FilteredOutput};
use crate::parser::CheckInfo;
use crate::{utils, Result};
use std::path::PathBuf;
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
    fn filter_output(output: &Output) -> FilteredOutput {
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
    fn check_info(raw_result: &str) -> Result<CheckInfo> {
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

        // Handle the rest of lines
        let mut code_string = String::new();
        let mut extracted_info = ExtractedCheckInfo::default();
        // regex to extract lint name from 'note: `#[warn(clippy::lint_name)]`'
        let note_regex = Regex::new(r"clippy::(?P<name>(\w+))")?;
        // regex to extract lint name from help message, which is from a url.
        let help_regex = Regex::new(r"rust-clippy/master/index.html#(?P<name>(\w+))")?;
        // A flag to mark whether the program is look for error code snippets or
        // not, otherwise it's possible that a suggestion code will be mistaken as
        // error code snippet.
        let mut looking_for_code = true;

        for line in lines {
            let trimmed = line.trim();
            // FIXME: this method is dumb, it's expensive and could cause false positive if
            // a string variable has 'help: ' inside of it.
            if trimmed.contains("help: ") {
                looking_for_code = false;
                update_info_from_help_or_note(trimmed, &help_regex, "help: ", &mut extracted_info);
            } else if trimmed.contains("note: ") {
                looking_for_code = false;
                update_info_from_help_or_note(trimmed, &note_regex, "note: ", &mut extracted_info);
            } else if looking_for_code {
                if let Some(code) = maybe_code_line(trimmed) {
                    code_string.push_str(code);
                }
            }
        }

        Ok(CheckInfo {
            file_path,
            defect_name: extracted_info.defect_name,
            tool: super::SupportedTool::Clippy,
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

fn update_info_from_help_or_note(
    line: &str,
    re: &Regex,
    split_by: &'static str,
    info: &mut ExtractedCheckInfo,
) {
    if let Some(msg) = line.split_once(split_by).map(|(_, s)| s) {
        info.additional_help.push_str(&format!("{msg}\n"));
    }
    if let Some(lint_name) = utils::regex_utils::get_named_match("name", re, line) {
        info.defect_name = lint_name;
    }
}

fn maybe_code_line(s: &str) -> Option<&str> {
    if let Some((num, code)) = s.split_once('|') {
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
        let note_regex = Regex::new(r"clippy::(?P<name>(\w+))").unwrap();
        let note_str = "= note: `#[warn(clippy::bool_comparison)]` on by default";

        let name = regex_utils::get_named_match("name", &note_regex, note_str);
        assert_eq!(name, Some("bool_comparison".to_string()));
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

        assert_eq!(
            maybe_code_line(code_line_1),
            Some("     if flag == true {}")
        );
        assert_eq!(maybe_code_line(code_line_2), None);
        assert_eq!(maybe_code_line(code_line_3), None);
    }
}
