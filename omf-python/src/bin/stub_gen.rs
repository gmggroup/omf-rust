use pyo3_stub_gen::Result;

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
    let mut pyi = std::fs::read_to_string("omf2.pyi").expect("Failed to read stub");
    pyi = format!("r\"\"\"\n{docs}\"\"\"\n\n{pyi}");
    std::fs::write("omf2.pyi", pyi).expect("Failed to re-write stub");
    Ok(())
}
