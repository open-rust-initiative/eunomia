use super::mock_dir;
use eunomia::tools::*;
use std::env::set_current_dir;

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
    let mut filtered = clippy_opt.filter_output(&output);

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
