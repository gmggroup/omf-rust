use pyo3_stub_gen::Result;
use regex::{Captures, RegexBuilder};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();
    let stub = omf2::stub_info()?;
    stub.generate()?;
    // Copy the crate docs from the top of lib.rs and add them to the stub.
    let mut docs = String::new();
    let lib_rs = std::fs::read_to_string("src/lib.rs").expect("Failed to read lib.rs");
    for line in lib_rs.lines() {
        if let Some(doc) = line.strip_prefix("//!") {
            docs.push_str(doc.strip_prefix(" ").unwrap_or(doc));
            docs.push('\n');
        }
    }
    let re = RegexBuilder::new(r"\[(.*?)\]\(crate::\w+::Py(\w+)\)")
        .build()
        .expect("Failed to build regex");
    let adjusted_docs = re.replace_all(&docs, |cap: &Captures| {
        format!(
            "[{text}](index.md#omf2.{dst})",
            text = cap.get(1).expect("missing match").as_str(),
            dst = cap.get(2).expect("missing match").as_str(),
        )
    });
    let mut pyi = std::fs::read_to_string("omf2.pyi").expect("Failed to read stub");
    pyi = format!("r\"\"\"\n{adjusted_docs}\"\"\"\n\n{pyi}");
    std::fs::write("omf2.pyi", pyi).expect("Failed to re-write stub");
    Ok(())
}
