use super::Deserialize;
use std::collections::HashSet;
use std::path::Path;

/// The user defined rules configuration.
///
/// User can specify which file to check, what compilation options to pass,
/// and what guidelines the checks will be referenced.
#[derive(Debug, Deserialize)]
pub struct RulesCfg<'ru> {
    pub file_path: &'ru Path,
    #[serde(default)]
    pub supplement_compilation_options: Option<&'ru str>,
    #[serde(default)]
    pub coding_guidelines: HashSet<&'ru str>,
}
