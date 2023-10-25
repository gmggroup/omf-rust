/// Library name.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// Library version.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// File format name.
pub const FORMAT_NAME: &str = "Open Mining Format";

/// File format extension.
pub const FORMAT_EXTENSION: &str = "omf";

/// File format major version number.
pub const FORMAT_VERSION_MAJOR: u32 = 2;

/// File format minor version number.
pub const FORMAT_VERSION_MINOR: u32 = 0;

/// File format pre-release version suffix.
///
/// This will always be `None` in release versions of the crate. Pre-release formats
/// may contain experimental changes so can't be opened in by release versions.
pub const FORMAT_VERSION_PRERELEASE: Option<&str> = Some("alpha.2");

/// Returns a string containing the file format version that this crate produces.
pub fn format_version() -> String {
    let mut v = format!("{FORMAT_VERSION_MAJOR}.{FORMAT_VERSION_MINOR}");
    if let Some(pre) = FORMAT_VERSION_PRERELEASE {
        v = format!("{v}-{pre}");
    }
    v
}

/// Returns a string containing the full name and version of the file format that this
/// crate produces.
pub fn format_full_name() -> String {
    format!("{} {}", FORMAT_NAME, format_version())
}

/// Returns the crate name and version.
pub fn crate_full_name() -> String {
    format!("{CRATE_NAME} {CRATE_VERSION}")
}
