use std::path::PathBuf;
use std::process::{Command, Output};

use lazy_static::lazy_static;
use regex::Regex;

use crate::parser::CheckInfo;
use crate::Result;

use super::{Checker, FilteredOutput};

#[derive(Default, Debug)]
pub struct MiriOpt {
    pub program: String,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
    pub cur_dir: PathBuf,
}

impl Checker for MiriOpt {
    fn check(&self) -> Result<Output> {
        let output = Command::new(&self.program)
            .current_dir(&self.cur_dir)
            .args(&self.args)
            .envs(self.envs.iter().map(|(k, v)| (k, v)))
            .output()?;
        Ok(output)
    }

    fn filter_output(&self, output: &Output) -> FilteredOutput {
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stdout: Vec<String> = stdout.trim().lines().map(String::from).collect();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        let mut stderr: Vec<String> = stderr.trim().split("\n\n").map(String::from).collect();
        // skip the Miri startup logs in the first item
        stderr[0] = stderr[0]
            .lines()
            .skip_while(|&s| !s.starts_with("error"))
            .collect::<Vec<_>>()
            .join("\n");
        // skip non-error items (e.g. isolate note items)
        stderr.retain(|s| s.contains('|'));

        FilteredOutput { stdout, stderr }
    }

    fn check_info(&self, raw_result: &str) -> Result<CheckInfo> {
        lazy_static! {
            static ref RE_HELP_INFO: Regex = Regex::new(r"error: (.*)").unwrap();
            static ref RE_LOCATION: Regex = Regex::new(r"--> ([^:]+):(\d+):(\d+)").unwrap();
            static ref RE_NOTE_HELP: Regex = Regex::new(r"(?m)^[\s=]*(?:help|note): (.*)").unwrap();
            static ref RE_CODE_LINE: Regex = Regex::new(r"(?m)^\s*\d+\s*\|\s*(.*)").unwrap();
        }

        let mut lines = raw_result.trim().lines();

        let help_info = lines.next().map_or(String::new(), |s| {
            let cap = RE_HELP_INFO.captures(s).unwrap();
            String::from(&cap[1])
        });

        let (file_path, begin_line, column) =
            lines
                .next()
                .map_or((None, None, None), |s| match RE_LOCATION.captures(s) {
                    None => (None, None, None),
                    Some(cap) => (
                        Some(PathBuf::from(&cap[1])),
                        cap[2].parse::<usize>().ok(),
                        cap[3].parse::<usize>().ok(),
                    ),
                });

        let code_lines: Vec<_> = RE_CODE_LINE
            .captures_iter(raw_result)
            .map(|cap| String::from(&cap[1]))
            .collect();

        let additional_help_lines: Vec<_> = RE_NOTE_HELP
            .captures_iter(raw_result)
            .map(|cap| String::from(&cap[1]))
            .collect();

        Ok(CheckInfo {
            file_path,
            defect_name: String::new(),
            tool: super::SupportedTool::Miri,
            begin_line,
            end_line: begin_line,
            column,
            code_string: code_lines.join("\n"),
            help_info,
            additional_help_info: additional_help_lines.join("\n"),
            guideline_list: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::tools::Checker;

    use super::MiriOpt;

    #[test]
    fn test_miri_check_info() {
        let input = "error: Undefined Behavior: Data race detected between (1) Write on thread `<unnamed>` and (2) Write on thread `<unnamed>` at alloc1. (2) just happened here
--> src/bin/data_race.rs:7:38
  |
7 |   let t2 = thread::spawn(|| unsafe { UNSAFE = 2 });
  |                                      ^^^^^^^^^^ Data race detected between (1) Write on thread `<unnamed>` and (2) Write on thread `<unnamed>` at alloc1. (2) just happened here
  |
help: and (1) occurred earlier here
--> src/bin/data_race.rs:6:38
  |
6 |   let t1 = thread::spawn(|| unsafe { UNSAFE = 1 });
  |                                      ^^^^^^^^^^
  = help: this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
  = help: see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
  = note: BACKTRACE (of the first span):
  = note: inside closure at src/bin/data_race.rs:7:38: 7:48";

        let miri_opt = MiriOpt::default();
        let check_info = miri_opt.check_info(input).unwrap();

        let expected_help_info = "Undefined Behavior: Data race detected between (1) Write on thread `<unnamed>` and (2) Write on thread `<unnamed>` at alloc1. (2) just happened here";
        assert_eq!(&check_info.help_info, expected_help_info);

        let expected_file_path = "src/bin/data_race.rs";
        assert_eq!(
            check_info.file_path,
            Some(PathBuf::from(expected_file_path))
        );
        assert_eq!(check_info.begin_line, Some(7));
        assert_eq!(check_info.column, Some(38));

        let expected_code_string = "let t2 = thread::spawn(|| unsafe { UNSAFE = 2 });
let t1 = thread::spawn(|| unsafe { UNSAFE = 1 });";
        assert_eq!(&check_info.code_string, expected_code_string);

        let expected_help_note = "and (1) occurred earlier here
this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
BACKTRACE (of the first span):
inside closure at src/bin/data_race.rs:7:38: 7:48";
        assert_eq!(&check_info.additional_help_info, expected_help_note);
    }
}
