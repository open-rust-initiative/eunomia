//! This module defining data that are related to the output of running the checks.

use std::path::PathBuf;

use super::guideline::GuidelineSummary;
use super::Serialize;
use crate::tools::SupportedTool;
use crate::Result;

/// Main output format of this program, containing a list of checked infomation.
#[derive(Debug, Serialize)]
pub struct Output {
    pub check_info: Vec<CheckInfo>,
}

impl Output {
    pub fn to_json_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn to_json_string_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Infomation about the checking result.
///
/// This struct basically contains detailed information such as:
/// - which file emits warning or errors
/// - what part of the file emits warning or errors
/// - what lint got triggered
/// - the detailed message of triggered warning/error
/// - and what guidelines it violates
/// - ...
///
/// Note: This struct needs to own the infomation it holds,
/// since we need to serialize them at the very end of the running stage
/// so we need
// FIXME: The default implementation is convenient when building the struct,
// but may cause an empty struct, or struct with missing required fields being
// serialized, switch to builder pattern when derive macro is available in the future.
#[derive(Debug, Serialize, Default)]
pub struct CheckInfo {
    pub file_path: Option<PathBuf>,
    pub defect_name: String,
    pub tool: SupportedTool,
    pub begin_line: Option<usize>,
    pub end_line: Option<usize>,
    pub column: Option<usize>,
    pub code_string: String,
    pub help_info: String,
    pub additional_help_info: String,
    pub guideline_list: Vec<GuidelineSummary>,
}

impl CheckInfo {
    pub fn defect_type(&self) -> DefectType {
        match self.tool {
            SupportedTool::Clippy => DefectType::ToolLint {
                is_rustc_lint: false,
                name: &self.defect_name,
            },
            SupportedTool::Rustc => DefectType::ToolLint {
                is_rustc_lint: true,
                name: &self.defect_name,
            },
            SupportedTool::Miri | SupportedTool::Sanitizer => {
                DefectType::KeyWord(&self.defect_name)
            }
        }
    }
}

impl Serialize for SupportedTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Type of error/warning that got triggered, with defect name borrowed from [`CheckInfo`].
///
/// - If the defection was detected by `clippy` or `rustc`,
/// this will be a [`DefectType::ToolLint`] type.
/// - But if it was detected by some tools that doesn't have clear identifications
/// for its error/warning, we can then use keyword to query the result,
/// therefore this will be a [`DefectType::KeyWord`] type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DefectType<'c> {
    ToolLint { is_rustc_lint: bool, name: &'c str },
    KeyWord(&'c str),
}
