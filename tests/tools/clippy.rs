use super::mock_dir;
use eunomia::{parser::CheckInfo, tools::*};
use std::{env::set_current_dir, path::PathBuf};

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

#[test]
fn clippy_default() {
    set_current_dir(mock_dir()).unwrap();

    let cmd = Command {
        app: "cargo",
        args: &["clippy"],
        envs: &[],
    };
    let clippy_opt = ClippyOpt::from_command(cmd);

    let output = clippy_opt.check().unwrap();
    let mut filtered = ClippyOpt::filter_output(&output);

    filtered.stderr.sort();
    let expected_output = vec![
      "error: range is out of bounds
  --> src/clippy.rs:12:19
   |
12 |     let _ = &x[2..9];
   |                   ^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing
   = note: `#[deny(clippy::out_of_bounds_indexing)]` on by default",
"error: this looks like you are trying to swap `_a` and `_b`
 --> src/clippy.rs:5:5
  |
5 | /     _a = _b;
6 | |     _b = _a;
  | |___________^ help: try: `std::mem::swap(&mut _a, &mut _b)`
  |
  = note: or maybe you should use `std::mem::replace`?
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#almost_swapped
  = note: `#[deny(clippy::almost_swapped)]` on by default",
"warning: `--x` could be misinterpreted as pre-decrement by C programmers, is usually a no-op
  --> src/clippy.rs:25:13
   |
25 |     let _ = --x;
   |             ^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#double_neg
   = note: `#[warn(clippy::double_neg)]` on by default",
r#"warning: comparison to empty slice
  --> src/clippy.rs:21:8
   |
21 |     if s == "" {}
   |        ^^^^^^^ help: using `is_empty` is clearer and more explicit: `s.is_empty()`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#comparison_to_empty
   = note: `#[warn(clippy::comparison_to_empty)]` on by default"#,
"warning: equality checks against true are unnecessary
  --> src/clippy.rs:18:8
   |
18 |     if flag == true {}
   |        ^^^^^^^^^^^^ help: try simplifying it as shown: `flag`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#bool_comparison
   = note: `#[warn(clippy::bool_comparison)]` on by default",
"warning: statement with no effect
  --> src/clippy.rs:11:5
   |
11 |     x[9];
   |     ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#no_effect
   = note: `#[warn(clippy::no_effect)]` on by default",
    ];

    assert_eq!(filtered.stdout, Vec::<String>::new());
    assert_eq!(filtered.stderr, expected_output);
}

#[test]
fn clippy_unified_output() {
    let output_1 = "error: range is out of bounds
  --> src/clippy.rs:12:19
   |
12 |     let _ = &x[2..9];
   |                   ^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing
   = note: `#[deny(clippy::out_of_bounds_indexing)]` on by default";

    let info = ClippyOpt::check_info(output_1).unwrap();

    let expected = CheckInfo {
      file_path: Some(PathBuf::from("src/clippy.rs")),
      tool: SupportedTool::Clippy,
      begin_line: Some(12),
      column: Some(19),
      help_info: "range is out of bounds".to_string(),
      defect_name: "out_of_bounds_indexing".to_string(),
      code_string: "     let _ = &x[2..9];".to_string(),
      additional_help_info: "for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing
`#[deny(clippy::out_of_bounds_indexing)]` on by default\n".to_string(),
      ..Default::default()
   };

    assert_eq_check_info(&info, &expected);
}

#[test]
fn clippy_unified_output_short() {
    let output_1 = "warning: package `mock` is missing `package.categories` metadata
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#cargo_common_metadata";

    let info = ClippyOpt::check_info(output_1).unwrap();

    let expected = CheckInfo {
      tool: SupportedTool::Clippy,
      help_info: "package `mock` is missing `package.categories` metadata".to_string(),
      defect_name: "cargo_common_metadata".to_string(),
      additional_help_info: "for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#cargo_common_metadata\n".to_string(),
      ..Default::default()
   };
    assert_eq_check_info(&info, &expected);
}
