use super::ModelType;

/// Errors specific to the OMF v1 conversion process.
///
/// Converted fails may also fail validation during conversion, as the checks in OMF v1 weren't
/// as strict.
#[derive(Debug, thiserror::Error)]
pub enum Omf1Error {
    /// Tried to load a file that is not in OMF1 format.
    #[error("this is not an OMF1 file")]
    NotOmf1,
    /// The OMF version is not supported.
    #[error("version '{version}' is not supported")]
    UnsupportedVersion { version: String },
    /// A record in the JSON data has the wrong type.
    #[error("wrong value type, found {found} when expecting {}", join(expected))]
    WrongType {
        found: ModelType,
        expected: &'static [ModelType],
    },
    /// A record is missing from the JSON data.
    #[error("item '{key}' is missing from the file")]
    MissingItem { key: String },
    /// Non-integer values from in what should be an integer array.
    #[error("an integer array was expected, but floating-point was found")]
    NonIntegerArray,
    /// An integer index is invalid.
    #[error("index {index} is outside the range 0 to 4294967295, or -1 for null categories")]
    IndexOutOfRange { index: i64 },
    /// Forwards `serde_json` errors when deserializing OMF1.
    #[error("JSON deserialization error: {0}")]
    DeserializationFailed(#[from] serde_json::Error),
}

fn join(types: &[ModelType]) -> String {
    match types {
        [] => "none".to_owned(),
        [t] => t.to_string(),
        [t, s] => format!("{t} or {s}"),
        [s @ .., t] => format!(
            "{}, or {t}",
            s.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
