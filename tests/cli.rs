mod common;
use common::setup;
use std::fs;
use std::process::Command;

#[test]
fn print_version() {
    setup(|cfg| {
        let res = Command::new(&cfg.bin_path).arg("-V").output().unwrap();
        let output_str = String::from_utf8(res.stdout).unwrap();
        assert_eq!(
            output_str,
            format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        );
    });
}

#[test]
fn run_with_rules_and_cli_paths() {
    setup(|cfg| {
        let expected_output_file = cfg.test_dir.join("data").join("output_lints_expected.json");
        let output_file = cfg.output_dir.join("output_lints.json");
        let st = Command::new(&cfg.bin_path)
            .args([
                "--rule-file",
                &format!("{}/data/default_rules.json", cfg.test_dir.display()),
                "--src-file",
                &format!("{}/mock/src/lib.rs", cfg.test_dir.display()),
                "--output",
                output_file.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        let expected_output = fs::read_to_string(expected_output_file).unwrap();
        let output_file_content = fs::read_to_string(output_file).unwrap();
        assert!(st.success());
        assert_eq!(expected_output, output_file_content);
    });
}
