//! This module contains definition/implementation for tools that this program
//! supports, such as `clippy`, `miri`, etc...
//!
//! All relavent definition should be declared here, just in case more tools will be
//! added or deleted in the future.

use std::fmt::Display;

#[derive(Debug, Default)]
pub enum SupportedTool {
    Clippy,
    #[default]
    Rustc,
    Miri,
    Sanitizer,
}

impl SupportedTool {
    // TODO: put this method in a derive macro
    pub fn all() -> Vec<Self> {
        vec![Self::Clippy, Self::Rustc, Self::Miri, Self::Sanitizer]
    }
}

// TODO: put this method in a derive macro
impl Display for SupportedTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SupportedTool::*;
        let str = match self {
            Clippy => "clippy",
            Rustc => "rustc",
            Miri => "miri",
            Sanitizer => "sanitizer",
        };
        f.write_str(str)
    }
}
