//! This module contains definitions of CodingGuidelines related data.

use super::{de, Deserialize, Serialize};
use crate::tools::SupportedTool;
use crate::{Error, Result};
use std::{hash::Hash, str::FromStr};

/// Contains a `Vec` of [`Guideline`] items.
#[derive(Debug, Deserialize)]
pub struct CodingGuidelines<'g> {
    #[serde(borrow)]
    pub coding_guidelines: Vec<Guideline<'g>>,
}

impl<'g> CodingGuidelines<'g> {
    pub fn from_json(s: &'g str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

/// Complete information about a coding guideline item,
/// including its relation with tools.
#[derive(Debug, Deserialize)]
pub struct Guideline<'g> {
    pub id: GuidelineID,
    pub name: &'g str,
    #[serde(default)]
    pub level: CheckLevel,
    pub tool: Vec<CheckTool<'g>>,
}

#[derive(Debug, Deserialize, PartialEq, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CheckLevel {
    Fatal,
    Severe,
    #[serde(alias = "normal")]
    #[default]
    Warn,
    Prompt,
    Info,
}

#[derive(Debug, Deserialize)]
pub struct CheckTool<'g> {
    /// Name of the tool, must be one of the [`SupportedTool`] variants.
    pub name: SupportedTool,
    /// The keyword of how we filter checking result to a specific guideline item.
    ///
    /// Could be a lint name, or specific keyword in a tool's output.
    pub ident: &'g str,
}

// Deserialize SupportedTool with its `FromStr` implementation.
impl<'de> Deserialize<'de> for SupportedTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

/// An unique identifier for a guideline item.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GuidelineID {
    /// A character representing the type of this guideline,
    /// such as 'P', 'G'.
    pub ty: char,
    /// The group where this guideline belongs,
    /// such as "TYP.INT".
    pub group: String,
    /// The index of this guideline inside its group,
    /// such as "01", "12".
    pub idx: String,
}

impl<T: AsRef<str>> PartialEq<T> for GuidelineID {
    fn eq(&self, other: &T) -> bool {
        if let Ok(id) = other.as_ref().parse::<Self>() {
            self == &id
        } else {
            false
        }
    }
}

impl FromStr for GuidelineID {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (maybe_ty, maybe_group_and_idx) = s
            .split_once('.')
            .ok_or(Error::InvalidGuidelineID(s.to_string()))?;
        let ty = if maybe_ty.len() == 1 {
            maybe_ty
                .chars()
                .next()
                .expect("unexpected error when extracting type char from guideline ID")
                .to_ascii_lowercase()
        } else {
            return Err(Error::InvalidGuidelineType(maybe_ty.to_string()));
        };
        let (group, idx) = maybe_group_and_idx
            .rsplit_once('.')
            .filter(|(g, i)| !g.is_empty() && !i.is_empty())
            .map(|(g, i)| (g.to_ascii_lowercase(), i.to_string()))
            .ok_or(Error::InvalidGuidelineID(s.to_string()))?;
        Ok(GuidelineID { ty, group, idx })
    }
}

impl ToString for GuidelineID {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.ty, self.group, self.idx)
    }
}

// Deserialize guideline ID with `FromStr` implementation.
impl<'de> Deserialize<'de> for GuidelineID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for GuidelineID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Basic information about a guideline item, including its id and name.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GuidelineSummary {
    pub id: GuidelineID,
    pub name: String,
}

impl Hash for GuidelineSummary {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::GuidelineID;

    #[test]
    fn good_guideline_id() {
        let id_1 = "P.Exam.Ple.01".parse::<GuidelineID>();
        let id_2 = "G.typ.arr.04".parse::<GuidelineID>();
        let id_3 = "P.VAR.CONST.02".parse::<GuidelineID>();

        assert_eq!(
            id_1,
            Ok(GuidelineID {
                ty: 'p',
                group: "exam.ple".to_string(),
                idx: "01".to_string()
            })
        );
        assert_eq!(
            id_2,
            Ok(GuidelineID {
                ty: 'g',
                group: "typ.arr".to_string(),
                idx: "04".to_string()
            })
        );
        assert_eq!(
            id_3,
            Ok(GuidelineID {
                ty: 'p',
                group: "var.const".to_string(),
                idx: "02".to_string()
            })
        );
    }

    #[test]
    fn guideline_id_with_various_length() {
        let id_1 = "P.Exam.Ple.With.Various.Len.999".parse::<GuidelineID>();
        let id_2 = "g.TEST.01".parse::<GuidelineID>();

        assert_eq!(
            id_1,
            Ok(GuidelineID {
                ty: 'p',
                group: "exam.ple.with.various.len".to_string(),
                idx: "999".to_string()
            })
        );
        assert_eq!(
            id_2,
            Ok(GuidelineID {
                ty: 'g',
                group: "test".to_string(),
                idx: "01".to_string()
            })
        );
    }

    #[test]
    fn bad_guideline_id() {
        // missing type (prefix)
        let id_1 = ".Exam.Ple.01".parse::<GuidelineID>();
        // missing index (postfix)
        let id_2 = "P.typ.int.i32.".parse::<GuidelineID>();
        // missing body
        let id_3 = "P.P".parse::<GuidelineID>();
        // type is not a single character
        let id_4 = "Guide.Typ.Arr.04".parse::<GuidelineID>();

        assert!(id_1.is_err());
        assert!(id_2.is_err());
        assert!(id_3.is_err());
        assert!(id_4.is_err());
    }

    #[test]
    fn guideline_id_cmp_with_str() {
        let id_1 = "p.typ.int.01";
        let id_2 = "G.TyP.INt.02";
        let id_3 = "G.TYP.INT.03";

        assert_eq!(id_1.parse::<GuidelineID>().unwrap(), id_1);
        assert_eq!(id_2.parse::<GuidelineID>().unwrap(), id_2.to_lowercase());
        assert_eq!(id_3.parse::<GuidelineID>().unwrap(), id_3.to_lowercase());
    }
}
