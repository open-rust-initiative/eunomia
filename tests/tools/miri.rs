use std::env;
use std::path::Path;

use eunomia::tools::{miri::MiriOpt, Checker};

#[test]
fn miri_default() {
    let mock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/mock-miri");
    let miri_opt = MiriOpt {
        program: "cargo".into(),
        args: vec![
            "miri".into(),
            "run".into(),
            "--bin".into(),
            "hello_world".into(),
        ],
        cur_dir: mock_path.clone(),
        ..Default::default()
    };

    let output = miri_opt.check().unwrap();
    let filtered = miri_opt.filter_output(&output);
    assert_eq!(filtered.stderr, Vec::<String>::new());
    assert_eq!(filtered.stdout, vec!["hello world!"]);

    let miri_opt = MiriOpt {
        program: "cargo".into(),
        args: vec![
            "miri".into(),
            "run".into(),
            "--bin".into(),
            "data_race".into(),
        ],
        cur_dir: mock_path,
        ..Default::default()
    };
    let output = miri_opt.check().unwrap();
    let filtered = miri_opt.filter_output(&output);

    let expected_stderr = vec![
        "error: Undefined Behavior: Data race detected between (1) Write on thread `<unnamed>` and (2) Write on thread `<unnamed>` at alloc1. (2) just happened here
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
  = note: inside closure at src/bin/data_race.rs:7:38: 7:48"
    ];
    assert_eq!(filtered.stdout, Vec::<String>::new());
    assert_eq!(filtered.stderr, expected_stderr);
}
