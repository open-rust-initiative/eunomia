use super::Deserialize;
use crate::Result;
use anyhow::bail;
use std::{collections::HashSet, path::Path};

pub trait ToolOpt<'ru> {
    fn enabled(&self) -> bool;
    fn compilation_opts(&self) -> Option<&'ru str>;
}

/// The configuration for checked rules.
#[derive(Debug, Deserialize)]
pub struct RulesCfg<'ru> {
    pub file_path: &'ru Path,
    #[serde(default)]
    pub supplement_compilation_options: Option<&'ru str>,
    #[serde(default)]
    pub lints: Lints<'ru>,
    #[serde(default)]
    pub miri: Miri<'ru>,
    #[serde(default)]
    pub sanitizer: Sanitizer<'ru>,
}

#[derive(Debug, Deserialize, Default)]
struct BasicToolOpt<'ru> {
    enable: bool,
    supplement_compilation_options: Option<&'ru str>,
}

/// Lint configurations, including clippy lints and rustc lints.
#[derive(Debug, Deserialize, Default)]
pub struct Lints<'ru> {
    #[serde(borrow, flatten)]
    opts: BasicToolOpt<'ru>,
    pub clippy: LintsOpt<'ru>,
    pub rustc: LintsOpt<'ru>,
}

/// Lints option.
///
/// Each group represent how the user want to handle the specified lints.
// TODO: Add custome deserializer to prevent intersection, but right now,
// we'll settle for calling a verifier method after deserializing is done.
#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct LintsOpt<'ru> {
    #[serde(borrow)]
    pub deny: Vec<&'ru str>,
    pub warn: Vec<&'ru str>,
    pub allow: Vec<&'ru str>,
}

impl LintsOpt<'_> {
    /// Verify the lints option to see if duplicate elements exist.
    ///
    /// For example, if user accidently wrote `clippy::all` in the deny list,
    /// the wrote it again in the warn list. This doesn't considered as error by clippy or
    /// rustc, but the result might differ due to which option gets passed at last.
    /// So, this method is to prevent that situation.
    pub fn verify(&self) -> Result<()> {
        let mut all = Vec::new();
        all.extend_from_slice(&self.deny);
        all.extend_from_slice(&self.warn);
        all.extend_from_slice(&self.allow);

        let hs_all: HashSet<&str> = HashSet::from_iter(all.iter().cloned());

        if all.len() != hs_all.len() {
            bail!("Lints configuration contains duplicate elements");
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Miri<'ru> {
    #[serde(borrow, flatten)]
    opts: BasicToolOpt<'ru>,
}

/// Supporting types of sanitizer,
/// see [sanitizer doc](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html)
/// for more information.
#[derive(Debug, Deserialize, PartialEq)]
pub enum SanitizerType {
    Address,
    Memory,
    Leak,
    Thread,
}

#[derive(Debug, Deserialize, Default)]
pub struct Sanitizer<'ru> {
    #[serde(borrow, flatten)]
    opts: BasicToolOpt<'ru>,
    #[serde(default = "default_sanitizer_types")]
    /// A list of supporting sanitizers, as defined as [`SanitizerType`].
    pub types: Vec<SanitizerType>,
}

fn default_sanitizer_types() -> Vec<SanitizerType> {
    vec![SanitizerType::Address]
}

// TODO: these impls are redundent, use derive macros.
impl<'ru> ToolOpt<'ru> for Lints<'ru> {
    fn enabled(&self) -> bool {
        self.opts.enable
    }
    fn compilation_opts(&self) -> Option<&'ru str> {
        self.opts.supplement_compilation_options
    }
}
impl<'ru> ToolOpt<'ru> for Miri<'ru> {
    fn enabled(&self) -> bool {
        self.opts.enable
    }
    fn compilation_opts(&self) -> Option<&'ru str> {
        self.opts.supplement_compilation_options
    }
}
impl<'ru> ToolOpt<'ru> for Sanitizer<'ru> {
    fn enabled(&self) -> bool {
        self.opts.enable
    }
    fn compilation_opts(&self) -> Option<&'ru str> {
        self.opts.supplement_compilation_options
    }
}
