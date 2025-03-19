// Generates some functions then calls cbindgen to generate the header file.

use std::fmt::Write;

fn main() {
    // cbindgen.toml contains the configuration.
    let crate_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR env var");
    let profile = std::env::var("PROFILE").expect("missing PROFILE env var");
    let mut config = cbindgen::Config::from_root_or_default(&crate_dir);
    config.after_includes = Some(defines());
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(format!("{crate_dir}/../target/{profile}/omf.h"));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src/");
}

fn defines() -> String {
    use omf::{
        CRATE_NAME, CRATE_VERSION, FORMAT_EXTENSION, FORMAT_NAME, FORMAT_VERSION_MAJOR,
        FORMAT_VERSION_MINOR, FORMAT_VERSION_PRERELEASE, format_full_name,
    };
    let mut defines = format!(
        "
// File format name and version numbers.
#define OMF_FORMAT_EXTENSION \"{FORMAT_EXTENSION}\"
#define OMF_FORMAT_NAME \"{FORMAT_NAME}\"
#define OMF_FORMAT_VERSION_MAJOR {FORMAT_VERSION_MAJOR}
#define OMF_FORMAT_VERSION_MINOR {FORMAT_VERSION_MINOR}
#define OMF_CRATE_NAME \"{CRATE_NAME}\"
#define OMF_CRATE_VERSION \"{CRATE_VERSION}\"
"
    );
    if let Some(pre) = FORMAT_VERSION_PRERELEASE {
        writeln!(
            &mut defines,
            "#define OMF_FORMAT_VERSION_PRERELEASE \"{pre}\""
        )
        .unwrap();
    }
    writeln!(
        &mut defines,
        "#define OMF_FORMAT_NAME_FULL \"{full}\"",
        full = format_full_name()
    )
    .unwrap();
    defines
}
