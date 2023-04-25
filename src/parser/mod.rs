mod guideline;
mod output_file;
mod rules_config;

use serde::{de, Deserialize, Serialize};

pub use guideline::*;
pub use output_file::{CheckInfo, Output};
pub use rules_config::*;

use crate::Result;

pub trait JsonStruct<'a> {
    fn deserialize(s: &'a str) -> Result<Self>
    where
        Self: de::Deserialize<'a>,
    {
        Ok(serde_json::from_str(s)?)
    }
}
