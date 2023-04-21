use super::mock_dir;
use anyhow::Result;
use eunomia::{parser::CheckInfo, tools::*};
use std::path::PathBuf;

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
fn lints_output_comparison(opt: LintsOpt, expected: &mut Vec<&str>) -> Result<()> {
    let output = opt.check()?;
    let mut filtered = opt.filter_output(&output);

    // `cargo check` and clippy (which runs cargo check) generate output as stderr
    filtered.stderr.sort();
    expected.sort();

    assert_eq!(filtered.stdout, Vec::<String>::new());
    assert_eq!(&filtered.stderr, expected);

    Ok(())
}

#[test]
fn clippy_lints_default() -> Result<()> {
    let opt = LintsOpt {
        program: "cargo".into(),
        args: vec!["clippy".into()],
        is_clippy: true,
        cur_dir: mock_dir().to_path_buf(),
        ..Default::default()
    };

    let mut expected_output = vec![
      "error: range is out of bounds
 --> src/lints.rs:9:19
  |
9 |     let _ = &x[2..9];
  |                   ^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#out_of_bounds_indexing
  = note: `#[deny(clippy::out_of_bounds_indexing)]` on by default",
      "error: this looks like you are trying to swap `_a` and `_b`
 --> src/lints.rs:5:5
  |
5 | /     _a = _b;
6 | |     _b = _a;
  | |___________^ help: try: `std::mem::swap(&mut _a, &mut _b)`
  |
  = note: or maybe you should use `std::mem::replace`?
  = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#almost_swapped
  = note: `#[deny(clippy::almost_swapped)]` on by default",
      "warning: `--x` could be misinterpreted as pre-decrement by C programmers, is usually a no-op
  --> src/lints.rs:22:13
   |
22 |     let _ = --x;
   |             ^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#double_neg
   = note: `#[warn(clippy::double_neg)]` on by default",
      r#"warning: comparison to empty slice
  --> src/lints.rs:18:8
   |
18 |     if s == "" {}
   |        ^^^^^^^ help: using `is_empty` is clearer and more explicit: `s.is_empty()`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#comparison_to_empty
   = note: `#[warn(clippy::comparison_to_empty)]` on by default"#,
      "warning: denote infinite loops with `loop { ... }`
  --> src/lints.rs:43:5
   |
43 |     while true { }
   |     ^^^^^^^^^^ help: use `loop`
   |
   = note: `#[warn(while_true)]` on by default",
      "warning: equality checks against true are unnecessary
  --> src/lints.rs:15:8
   |
15 |     if flag == true {}
   |        ^^^^^^^^^^^^ help: try simplifying it as shown: `flag`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#bool_comparison
   = note: `#[warn(clippy::bool_comparison)]` on by default",
      "warning: function `dead_fn` is never used
  --> src/lints.rs:36:4
   |
36 | fn dead_fn() { }
   |    ^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default",
      "warning: private type `lints::rustc_warning_lints::Priv` in public interface (error E0446)
  --> src/lints.rs:47:9
   |
47 |         pub fn f(_: Priv) { }
   |         ^^^^^^^^^^^^^^^^^
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #34537 <https://github.com/rust-lang/rust/issues/34537>
   = note: `#[warn(private_in_public)]` on by default",
      "warning: this lifetime isn't used in the function definition
  --> src/lints.rs:62:42
   |
62 | pub fn rustc_allow_lint_unused_lifetimes<'a>() { }
   |                                          ^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#extra_unused_lifetimes
   = note: `#[warn(clippy::extra_unused_lifetimes)]` on by default",
      "warning: unused variable: `x`
  --> src/lints.rs:41:9
   |
41 |     let x = 5;
   |         ^ help: if this is intentional, prefix it with an underscore: `_x`
   |
   = note: `#[warn(unused_variables)]` on by default",
    ];

    lints_output_comparison(opt, &mut expected_output)
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
fn rustc_lints_default() -> Result<()> {
    let cargo_cmd = LintsOpt {
        program: "cargo".into(),
        args: vec!["check".into()],
        is_clippy: false,
        cur_dir: mock_dir().to_path_buf(),
        ..Default::default()
    };
    let rustc_cmd = LintsOpt {
        program: "rustc".into(),
        cur_dir: mock_dir().to_path_buf(),
        args: vec![
            "src/lints.rs".into(),
            "--crate-type".into(),
            "lib".into(),
            "--out-dir".into(),
            "target/".into(),
        ],
        is_clippy: false,
        ..Default::default()
    };
    let mut expected = vec![
        "warning: denote infinite loops with `loop { ... }`
  --> src/lints.rs:43:5
   |
43 |     while true { }
   |     ^^^^^^^^^^ help: use `loop`
   |
   = note: `#[warn(while_true)]` on by default",
        "warning: unused variable: `x`
  --> src/lints.rs:41:9
   |
41 |     let x = 5;
   |         ^ help: if this is intentional, prefix it with an underscore: `_x`
   |
   = note: `#[warn(unused_variables)]` on by default",
        "warning: private type `Priv` in public interface (error E0446)
  --> src/lints.rs:47:9
   |
47 |         pub fn f(_: Priv) { }
   |         ^^^^^^^^^^^^^^^^^
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #34537 <https://github.com/rust-lang/rust/issues/34537>
   = note: `#[warn(private_in_public)]` on by default",
        "warning: function `dead_fn` is never used
  --> src/lints.rs:36:4
   |
36 | fn dead_fn() { }
   |    ^^^^^^^
   |
   = note: `#[warn(dead_code)]` on by default",
    ];

    lints_output_comparison(cargo_cmd, &mut expected)?;
    lints_output_comparison(rustc_cmd, &mut expected)
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
