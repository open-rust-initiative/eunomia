use eunomia::parser::{CheckInfo, GuidelineSummary, Output};
use eunomia::tools::SupportedTool;
use std::path::PathBuf;

#[test]
fn se_check_info_single_entity() {
    let check_info_list = vec![CheckInfo {
        file_path: PathBuf::from("./src/main.rs"),
        tool: SupportedTool::Clippy,
        defect_name: "exhaustive_enums".to_string(),
        begin_line: Some(15),
        column: Some(23),
        code_string: Some(String::new()),
        help_info: Some(String::new()),
        additional_help_info: Some(String::new()),
        guideline_list: vec![GuidelineSummary {
            id: "P.VAR.01".parse().unwrap(),
            name: "ssss".to_string(),
        }],
        ..Default::default()
    }];
    let output = Output {
        check_info: check_info_list,
    };

    let expected_json = r#"{
  "check_info": [
    {
      "file_path": "./src/main.rs",
      "defect_name": "exhaustive_enums",
      "tool": "clippy",
      "begin_line": 15,
      "end_line": null,
      "column": 23,
      "code_string": "",
      "help_info": "",
      "additional_help_info": "",
      "guideline_list": [
        {
          "id": "p.var.01",
          "name": "ssss"
        }
      ]
    }
  ]
}"#;

    let output_string = output.to_json_string_pretty();
    assert!(output_string.is_ok());
    assert_eq!(output_string.unwrap(), expected_json);
}

#[test]
fn se_check_info_multiple_entities() {
    let check_info_list = vec![
        CheckInfo {
            file_path: PathBuf::from("./src/main.rs"),
            tool: SupportedTool::Clippy,
            defect_name: "exhaustive_enums".to_string(),
            begin_line: Some(15),
            end_line: Some(18),
            column: Some(23),
            code_string: Some("xxx {\n\n\n xxa }".to_string()),
            help_info: Some(String::new()),
            additional_help_info: Some(String::new()),
            guideline_list: vec![GuidelineSummary {
                id: "P.VAR.01".parse().unwrap(),
                name: "ssss".to_string(),
            }],
        },
        CheckInfo {
            file_path: PathBuf::from("./src/lib.rs"),
            defect_name: "dead_code".to_string(),
            begin_line: Some(20),
            column: Some(8),
            code_string: Some("let x = 1;".to_string()),
            guideline_list: vec![
                GuidelineSummary {
                    id: "g.exam.ple.01".parse().unwrap(),
                    name: "don't have dead_code".to_string(),
                },
                GuidelineSummary {
                    id: "p.exam.ple.02".parse().unwrap(),
                    name: "useless variables".to_string(),
                },
            ],
            ..Default::default()
        },
        CheckInfo {
            file_path: PathBuf::from("./src/something.rs"),
            defect_name: "memory_leak".to_string(),
            tool: SupportedTool::Sanitizer,
            guideline_list: vec![GuidelineSummary {
                id: "g.exam.ple.03".parse().unwrap(),
                name: "free memory allocation after use".to_string(),
            }],
            ..Default::default()
        },
    ];

    let expected_json = r#"{
  "check_info": [
    {
      "file_path": "./src/main.rs",
      "defect_name": "exhaustive_enums",
      "tool": "clippy",
      "begin_line": 15,
      "end_line": 18,
      "column": 23,
      "code_string": "xxx {\n\n\n xxa }",
      "help_info": "",
      "additional_help_info": "",
      "guideline_list": [
        {
          "id": "p.var.01",
          "name": "ssss"
        }
      ]
    },
    {
      "file_path": "./src/lib.rs",
      "defect_name": "dead_code",
      "tool": "rustc",
      "begin_line": 20,
      "end_line": null,
      "column": 8,
      "code_string": "let x = 1;",
      "help_info": null,
      "additional_help_info": null,
      "guideline_list": [
        {
          "id": "g.exam.ple.01",
          "name": "don't have dead_code"
        },
        {
          "id": "p.exam.ple.02",
          "name": "useless variables"
        }
      ]
    },
    {
      "file_path": "./src/something.rs",
      "defect_name": "memory_leak",
      "tool": "sanitizer",
      "begin_line": null,
      "end_line": null,
      "column": null,
      "code_string": null,
      "help_info": null,
      "additional_help_info": null,
      "guideline_list": [
        {
          "id": "g.exam.ple.03",
          "name": "free memory allocation after use"
        }
      ]
    }
  ]
}"#;

    let op_str = Output {
        check_info: check_info_list,
    }
    .to_json_string_pretty();

    assert!(op_str.is_ok());
    assert_eq!(op_str.unwrap(), expected_json);
}

#[test]
fn se_empty_check_info() {
    // FIXME: this should not be a valid output,
    // after we have derive macro for CheckInfo, all non-option
    // fields should be required, and will throw error when not provided.
    let op_str = Output {
        check_info: vec![CheckInfo::default()],
    }
    .to_json_string_pretty();

    let expected_json = r#"{
  "check_info": [
    {
      "file_path": "",
      "defect_name": "",
      "tool": "rustc",
      "begin_line": null,
      "end_line": null,
      "column": null,
      "code_string": null,
      "help_info": null,
      "additional_help_info": null,
      "guideline_list": []
    }
  ]
}"#;

    assert!(op_str.is_ok());
    assert_eq!(op_str.unwrap(), expected_json);
}
