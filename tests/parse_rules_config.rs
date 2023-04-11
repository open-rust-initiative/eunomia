use std::path::Path;

use eunomia::parser::*;

#[test]
fn de_normal_rules() {
    let rule_input = r#"
{
    "file_path": "./src/main.rs",
    "supplement_compilation_options": "-I../a/b/c/d sources=../e/f/g/h.rs",
    "lints": {
        "enable": true,
        "supplement_compilation_options": "",
        "clippy": {
            "deny": [],
            "warn": [
                "all",
                "pedantic"
            ],
            "allow": [ "cargo" ]
        },
        "rustc": {
            "deny": [],
            "warn": [],
            "allow": []
        }
    },
    "miri": {
        "enable": true,
        "supplement_compilation_options": ""
    },
    "sanitizer": {
        "enable": true,
        "supplement_compilation_options": ""
    }
}
"#;

    let test_cfg: Result<RulesCfg, _> = serde_json::from_str(rule_input);
    assert!(test_cfg.is_ok());

    let cfg = test_cfg.unwrap();
    let compilation_options = Some("");
    assert_eq!(cfg.file_path, Path::new("./src/main.rs"));
    assert_eq!(
        cfg.supplement_compilation_options,
        Some("-I../a/b/c/d sources=../e/f/g/h.rs")
    );
    assert!(cfg.lints.enabled());
    assert_eq!(cfg.lints.compilation_opts(), compilation_options);
    assert_eq!(
        cfg.lints.clippy,
        LintsOpt {
            deny: Vec::new(),
            warn: Vec::from(["all", "pedantic"]),
            allow: Vec::from(["cargo"])
        }
    );
    assert_eq!(cfg.lints.rustc, LintsOpt::default());
    assert!(cfg.miri.enabled());
    assert!(cfg.sanitizer.enabled());
    assert_eq!(cfg.miri.compilation_opts(), compilation_options);
    assert_eq!(cfg.sanitizer.compilation_opts(), compilation_options);
    assert_eq!(cfg.sanitizer.types, [SanitizerType::Address]);
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
    assert!(!cfg.lints.enabled());
    assert!(cfg.lints.compilation_opts().is_none());
    assert_eq!(cfg.lints.clippy, LintsOpt::default());
    assert_eq!(cfg.lints.rustc, LintsOpt::default());
    assert!(!cfg.miri.enabled());
    assert!(!cfg.sanitizer.enabled());
    assert!(cfg.miri.compilation_opts().is_none());
    assert!(cfg.sanitizer.compilation_opts().is_none());
    assert!(cfg.sanitizer.types.is_empty());
}

#[test]
fn de_rules_with_duplicate_lints() {
    let rule_str = r#"
    {
        "file_path": "./src/main.rs",
        "lints": {
            "enable": true,
            "clippy": {
                "warn": [
                    "all",
                    "pedantic"
                ],
                "allow": [ "cargo" ],
                "deny": [ "all" ]
            },
            "rustc": {
                "deny": ["unused"],
                "warn": ["all"],
                "allow": []
            }
        }
    }
    "#;

    let test_cfg: Result<RulesCfg, _> = serde_json::from_str(rule_str);
    assert!(test_cfg.is_ok());
    let cfg = test_cfg.unwrap();
    // clippy lints option contains dup
    assert!(cfg.lints.clippy.verify().is_err());
    // rustc lints option does not contains dup
    assert!(cfg.lints.rustc.verify().is_ok());
}
