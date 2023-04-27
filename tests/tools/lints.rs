use super::mock_dir;
use crate::common::setup;
use anyhow::Result;
use eunomia::{parser::CheckInfo, tools::*};
use std::{fs, path::PathBuf};

/// Manually compare two check info, without the need of impl PartialEq for the entire
/// struct just for the sake of tests, and provide more details in which member doesn't match.
fn assert_eq_check_info(lhs: &CheckInfo, rhs: &CheckInfo) {
    assert_eq!(lhs.file_path, rhs.file_path, "file_path");
    assert_eq!(lhs.begin_line, rhs.begin_line, "begin_line");
    assert_eq!(lhs.end_line, rhs.end_line, "end_line");
    assert_eq!(lhs.column, rhs.column, "column");
    assert_eq!(lhs.defect_name, rhs.defect_name, "defect_name");
    assert_eq!(lhs.help_info, rhs.help_info, "help_info");
    assert_eq!(
        lhs.additional_help_info, rhs.additional_help_info,
        "additional_help_info"
    );
    assert_eq!(lhs.code_string, rhs.code_string, "code_string");
    assert_eq!(lhs.tool, rhs.tool, "tool");
    assert_eq!(lhs.guideline_list, rhs.guideline_list, "guideline_list");
}

// FIXME: this comparison is extremely unefficient,
// each time the mocked crate got changed, we have to manually modify the expected
// output to pass the test, can we use something similar to clippy's `dev bless` mechanism?
fn lints_output_comparison(opt: LintsOpt, expected_file: PathBuf) -> Result<()> {
    let output = opt.check()?;
    let mut filtered = opt.filter_output(&output);

    // `cargo check` and clippy (which runs cargo check) generate output as stderr
    filtered.stderr.sort();
    let actual_content = filtered.stderr.join("\n");

    // remove actual file path in stderr
    // absolute file paths are troublesome when running ci.
    let actual_content_masked = actual_content.lines().map(|s| {
        if s.trim().starts_with("-->") {
            if let Some((tabs, _)) = s.split_once("-->") {
                return format!("{tabs}--> _");
            }
        }
        s.to_string()
    }).collect::<Vec<_>>().join("\n");
    let expected_content = fs::read_to_string(expected_file).unwrap_or_default();

    assert_eq!(filtered.stdout, Vec::<String>::new());
    assert_eq!(actual_content_masked, expected_content);

    Ok(())
}

#[test]
fn clippy_lints_default() {
    setup(|cfg| {
        let opt = LintsOpt {
            use_cargo: true,
            is_clippy: true,
            path: mock_dir().to_path_buf(),
            ..Default::default()
        };

        lints_output_comparison(
            opt,
            cfg.test_dir
                .join("data")
                .join("clippy_lints_default_expected.stderr"),
        )
        .unwrap();
    });
}

#[test]
fn clippy_unified_output() {
    let output_1 = "error: range is out of bounds
  --> src/lints.rs:12:19
   |
12 |     let _ = &x[2..9];
   |                   ^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing
   = note: `#[deny(clippy::out_of_bounds_indexing)]` on by default";

    let info = LintsOpt {
        is_clippy: true,
        ..Default::default()
    }
    .check_info(output_1)
    .unwrap();

    let expected = CheckInfo {
      file_path: Some(PathBuf::from("src/lints.rs")),
      tool: SupportedTool::Clippy,
      begin_line: Some(12),
      column: Some(19),
      help_info: "range is out of bounds".to_string(),
      defect_name: "clippy::out_of_bounds_indexing".to_string(),
      code_string: "    let _ = &x[2..9];".to_string(),
      additional_help_info:  "\
        help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing\n\
        note: `#[deny(clippy::out_of_bounds_indexing)]` on by default\n".to_string(),
      ..Default::default()
   };

    assert_eq_check_info(&info, &expected);
}

#[test]
fn clippy_unified_output_short() {
    let output_1 = "warning: package `mock` is missing `package.categories` metadata
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#cargo_common_metadata";

    let opt = LintsOpt {
        is_clippy: true,
        ..Default::default()
    };
    let info = opt.check_info(output_1).unwrap();

    let expected = CheckInfo {
      tool: SupportedTool::Clippy,
      help_info: "package `mock` is missing `package.categories` metadata".to_string(),
      defect_name: "cargo_common_metadata".to_string(),
      additional_help_info: "help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#cargo_common_metadata\n".to_string(),
      ..Default::default()
   };
    assert_eq_check_info(&info, &expected);
}

#[test]
fn rustc_lints_default() {
    setup(|cfg| {
        let rustc_cmd = LintsOpt {
            path: mock_dir().join("src").join("lib.rs"),
            ..Default::default()
        };
        lints_output_comparison(
            rustc_cmd,
            cfg.test_dir
                .join("data")
                .join("rustc_lints_default_expected.stderr"),
        )
        .unwrap();
    });
}

#[test]
fn rustc_lints_using_cargo() {
    setup(|cfg| {
        let cargo_cmd = LintsOpt {
            use_cargo: true,
            path: mock_dir().to_path_buf(),
            ..Default::default()
        };
        lints_output_comparison(
            cargo_cmd,
            cfg.test_dir
                .join("data")
                .join("rustc_lints_using_cargo_expected.stderr"),
        )
        .unwrap();
    });
}

#[test]
fn rustc_unified_output() {
    let output = "warning: denote infinite loops with `loop { ... }`
  --> src/lints.rs:43:5
   |
43 |     while true { }
   |     ^^^^^^^^^^ help: use `loop`
   |
   = note: `#[warn(while_true)]` on by default";

    let opt = LintsOpt {
        is_clippy: false,
        ..Default::default()
    };
    let info = opt.check_info(output).unwrap();

    let expected = CheckInfo {
        file_path: Some(PathBuf::from("src/lints.rs")),
        defect_name: "while_true".to_string(),
        tool: SupportedTool::Rustc,
        begin_line: Some(43),
        column: Some(5),
        code_string: "    while true { }".to_string(),
        help_info: "denote infinite loops with `loop { ... }`".to_string(),
        additional_help_info: "note: `#[warn(while_true)]` on by default\n".to_string(),
        ..Default::default()
    };
    assert_eq_check_info(&info, &expected);
}

#[test]
fn rustc_unified_output_custom() {
    let output = "warning: identifier contains non-ASCII characters
  --> src/lints.rs:53:9
   |
53 |     let _变量 = 1;
   |         ^^^^^
   |
   = note: requested on the command line with `-W non-ascii-idents`";

    let info = LintsOpt {
        is_clippy: false,
        ..Default::default()
    }
    .check_info(output)
    .unwrap();

    let expected = CheckInfo {
        file_path: Some(PathBuf::from("src/lints.rs")),
        defect_name: "non-ascii-idents".to_string(),
        tool: SupportedTool::Rustc,
        begin_line: Some(53),
        column: Some(9),
        code_string: "    let _变量 = 1;".to_string(),
        help_info: "identifier contains non-ASCII characters".to_string(),
        additional_help_info: "note: requested on the command line with `-W non-ascii-idents`\n"
            .to_string(),
        ..Default::default()
    };
    assert_eq_check_info(&info, &expected);
}
