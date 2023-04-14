use eunomia::parser::{CheckLevel, CheckTool, CodingGuidelines, Guideline};
use eunomia::tools::SupportedTool;

/// Manually comparing two CodingGuidelines struct without impl PartialEq for that
/// struct, which we probably won't use outside of test cases.
fn guidlines_eq(lhs: &CodingGuidelines, rhs: &CodingGuidelines) {
    assert_eq!(lhs.coding_guidelines.len(), rhs.coding_guidelines.len());

    for i in 0..lhs.coding_guidelines.len() {
        let lhs_gl = &lhs.coding_guidelines[i];
        let rhs_gl = &rhs.coding_guidelines[i];

        assert_eq!(lhs_gl.id, rhs_gl.id);
        assert_eq!(lhs_gl.name, rhs_gl.name);
        assert_eq!(lhs_gl.level, rhs_gl.level);
        assert_eq!(lhs_gl.tool.len(), rhs_gl.tool.len());

        for j in 0..lhs_gl.tool.len() {
            let lhs_tool = &lhs_gl.tool[j];
            let rhs_tool = &rhs_gl.tool[j];

            assert_eq!(lhs_tool.name, rhs_tool.name);
            assert_eq!(lhs_tool.ident, rhs_tool.ident);
        }
    }
}

#[test]
fn regular_guidelines() {
    let gl_str = r#"
{
    "coding_guidelines": [
        {
            "id": "P.VAR.01",
            "name": "xxxxx",
            "level": "warn",
            "tool": [
                {
                    "name": "clippy",
                    "ident": "some_clippy_lint"
                }
            ]
        },
        {
            "id": "P.VAR.02",
            "name": "yyyyy",
            "level": "info",
            "tool": [
                {
                    "name": "rustc",
                    "ident": "some_rustc_lint"
                }
            ]
        },
        {
            "id": "P.VAR.03",
            "name": "aaaaa",
            "level": "fatal",
            "tool": [
                {
                    "name": "miri",
                    "ident": "some_miri_output_keyword"
                },
                {
                    "name": "sanitizer",
                    "ident": "some_sanitizer_output_keyword"
                }
            ]
        },
        {
            "id": "P.VAR.04",
            "name": "bbbbb",
            "level": "severe",
            "tool": [
                {
                    "name": "sanitizer",
                    "ident": "some_sanitizer_output_keyword"
                }
            ]
        }
    ]
}
    "#;

    let expected = CodingGuidelines {
        coding_guidelines: vec![
            Guideline {
                id: "p.var.01".parse().unwrap(),
                name: "xxxxx",
                level: CheckLevel::Warn,
                tool: vec![CheckTool {
                    name: SupportedTool::Clippy,
                    ident: "some_clippy_lint",
                }],
            },
            Guideline {
                id: "p.var.02".parse().unwrap(),
                name: "yyyyy",
                level: CheckLevel::Info,
                tool: vec![CheckTool {
                    name: SupportedTool::Rustc,
                    ident: "some_rustc_lint",
                }],
            },
            Guideline {
                id: "p.var.03".parse().unwrap(),
                name: "aaaaa",
                level: CheckLevel::Fatal,
                tool: vec![
                    CheckTool {
                        name: SupportedTool::Miri,
                        ident: "some_miri_output_keyword",
                    },
                    CheckTool {
                        name: SupportedTool::Sanitizer,
                        ident: "some_sanitizer_output_keyword",
                    },
                ],
            },
            Guideline {
                id: "p.var.04".parse().unwrap(),
                name: "bbbbb",
                level: CheckLevel::Severe,
                tool: vec![CheckTool {
                    name: SupportedTool::Sanitizer,
                    ident: "some_sanitizer_output_keyword",
                }],
            },
        ],
    };

    let gl = serde_json::from_str::<CodingGuidelines>(gl_str);
    assert!(gl.is_ok());
    guidlines_eq(&gl.unwrap(), &expected);
}

#[test]
fn default_check_level() {
    let gl_str = r#"
{
    "coding_guidelines": [
        {
            "id": "P.VAR.01",
            "name": "xxxxx",
            "tool": []
        }
    ]
}
    "#;

    assert_eq!(
        serde_json::from_str::<CodingGuidelines>(gl_str)
            .unwrap()
            .coding_guidelines[0]
            .level,
        CheckLevel::Warn
    );
}

#[test]
fn empty_coding_guidelines() {
    let gl_str = r#"
    {
        "coding_guidelines": []
    }
    "#;

    assert!(serde_json::from_str::<CodingGuidelines>(gl_str)
        .unwrap()
        .coding_guidelines
        .is_empty());
}

#[test]
fn guideline_missing_fields() {
    // Missing id
    let gl_1_str = r#"
    {
        "coding_guidelines": [
            {
                "name": "xxxxx",
                "tool": []
            }
        ]
    }
    "#;

    // Missing name
    let gl_2_str = r#"
    {
        "coding_guidelines": [
            {
                "id": "g.exam.ple.01",
                "tool": []
            }
        ]
    }
    "#;

    // Missing checking tool
    let gl_3_str = r#"
    {
        "coding_guidelines": [
            {
                "name": "xxxxx",
                "id": "g.exam.ple.01",
            }
        ]
    }
    "#;

    assert!(CodingGuidelines::from_json(gl_1_str).is_err());
    assert!(CodingGuidelines::from_json(gl_2_str).is_err());
    assert!(CodingGuidelines::from_json(gl_3_str).is_err());
}

#[test]
fn faulty_guidelines() {
    // Faulty id
    let gl_1_str = r#"
    {
        "coding_guidelines": [
            {
                "id": "."
                "name": "xxxxx",
                "tool": [
                    {
                        "name": "rustc",
                        "ident": "some_rustc_lint"
                    }
                ]
            }
        ]
    }
    "#;

    // Faulty tool name
    let gl_2_str = r#"
    {
        "coding_guidelines": [
            {
                "id": "."
                "name": "xxxxx",
                "tool": [
                    {
                        "name": "non-exist-tool",
                        "ident": "some_rustc_lint"
                    }
                ]
            }
        ]
    }
    "#;

    assert!(CodingGuidelines::from_json(gl_1_str).is_err());
    assert!(CodingGuidelines::from_json(gl_2_str).is_err());
}
