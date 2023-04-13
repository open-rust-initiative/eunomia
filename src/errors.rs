use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
/// Generic error types defining possible error outputs throughout this whole program.
#[non_exhaustive]
pub enum Error {
    InvalidGuidelineID(String),
    /// This is different than [`Error::InvalidGuidelineID`], this error should be
    /// thrown when the type of a guideline is not a single character.
    InvalidGuidelineType(String),
    ParseUnsupportedEnumVariant(&'static str, String, Vec<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        let msg = match self {
            InvalidGuidelineID(id) => format!(
                "'{id}' is not a valid guideline ID. A valid ID should looks like: \"G.Exam.Ple.01\""
            ),
            InvalidGuidelineType(ty) => format!(
                "'{ty}' is not a valid guideline type. A valid type should be a single character such as 'P' or 'G'"
            ),
            ParseUnsupportedEnumVariant(name, variant, all) => format!(
                "'{variant}' is not a valid variant of {name}. Supported variants are: [{}]",
                all.join(", ")
            ),
        };
        f.write_str(&msg)
    }
}
