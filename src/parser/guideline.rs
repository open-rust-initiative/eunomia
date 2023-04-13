//! This module contains definitions of CodingGuidelines related data.

use super::{de, Deserialize, Serialize};
use crate::Error;
use std::{hash::Hash, str::FromStr};

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
#[derive(Debug, Deserialize, Serialize)]
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
}
