use std::path::Path;
use std::collections::HashSet;

use eunomia::parser::*;

#[test]
fn de_normal_rules() {
    let rule_input = r#"
{
    "file_path": "./src/main.rs",
    "supplement_compilation_options": "-I../a/b/c/d sources=../e/f/g/h.rs",
    "coding_guidelines": [
        "P.Exam.Ple.01",
        "P.Exam.Ple.02",
        "P.Exam.Ple.03",
        "G.Exam.Ple.01",
        "G.Exam.Ple.02"
    ]
}
"#;

    let test_cfg: Result<RulesCfg, _> = serde_json::from_str(rule_input);
    assert!(test_cfg.is_ok());

    let cfg = test_cfg.unwrap();
    assert_eq!(cfg.file_path, Path::new("./src/main.rs"));
    assert_eq!(
        cfg.supplement_compilation_options,
        Some("-I../a/b/c/d sources=../e/f/g/h.rs")
    );
    assert_eq!(
        cfg.coding_guidelines,
        HashSet::from([
            "P.Exam.Ple.01".parse().unwrap(),
            "P.Exam.Ple.02".parse().unwrap(),
            "P.Exam.Ple.03".parse().unwrap(),
            "G.Exam.Ple.01".parse().unwrap(),
            "G.Exam.Ple.02".parse().unwrap(),
        ])
    );
}

#[test]
fn de_default_rules() {
    let rule_str = r#"
{
    "file_path": "./src/main.rs"
}"#;

    let test_cfg: Result<RulesCfg, _> = serde_json::from_str(rule_str);
    assert!(test_cfg.is_ok());

    let cfg = test_cfg.unwrap();
    assert_eq!(cfg.file_path, Path::new("./src/main.rs"));
    assert!(cfg.supplement_compilation_options.is_none());
    assert!(cfg.coding_guidelines.is_empty());
}

#[test]
fn de_rules_missing_path() {
    let rule_str = r#"
    {
        "supplement_compilation_options": "-I../a/b/c/d sources=../e/f/g/h.rs"
    }"#;

    let test_cfg: Result<RulesCfg, _> = serde_json::from_str(rule_str);
    assert!(test_cfg.is_err());
}
