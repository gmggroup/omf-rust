#![cfg(feature = "omf1")]

use std::{fs::read_dir, path::Path, time::Instant};

use omf::{error::Error, file::Reader, omf1::Converter};

#[test]
fn convert_omf1() {
    let output_path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("test_proj.2.omf");
    let converter = Converter::new();
    let warnings = converter
        .convert_open("tests/omf1/test_proj.omf", &output_path)
        .unwrap();
    let warning_strings: Vec<_> = warnings.into_iter().map(|p| p.to_string()).collect();
    assert_eq!(
        warning_strings,
        vec!["Warning: 'Project::elements[..]::name' contains duplicate of \"\", inside ''"]
    );
    let project = Reader::open(&output_path).unwrap().project().unwrap().0;
    let metadata = serde_json::to_string_pretty(&project.metadata).unwrap();
    dbg!(&metadata);
    assert!(metadata.starts_with(
        r#"{
  "OMF1 conversion": {
    "by": "omf 0.1.0-beta.1",
    "from": "OMF-v0.9.0",
    "on": "#
    ));
    assert!(metadata.ends_with(
        r#"Z"
  },
  "date_created": "2017-10-04T21:46:17Z",
  "date_modified": "2017-10-04T21:46:17Z"
}"#
    ));
}

#[ignore = "requires local files"]
#[test]
fn convert_external_files() {
    let dirs = &[
        Path::new("C:/Work/data/OMF"),
        Path::new("C:/Work/data/OMF/Geosoft Voxel OMF"),
    ];

    let mut converter = Converter::new();
    let mut limits = omf::file::Limits::default();
    limits.json_bytes = None;
    converter.set_limits(limits);
    let mut success = true;
    for dir in dirs {
        for file in read_dir(dir).unwrap().map(Result::unwrap) {
            let name = file.file_name().into_string().unwrap();
            if name.ends_with(".omf") {
                let len = file.metadata().unwrap().len();
                let size = if len > 1024 * 1024 {
                    format!("{:.1} MB", (len as f64) / 1024.0 / 1024.0)
                } else {
                    format!("{:.1} KB", (len as f64) / 1024.0)
                };
                println!("{name} ({size})");
                let output = Path::new(env!("CARGO_TARGET_TMPDIR")).join(file.file_name());
                let start = Instant::now();
                match converter.convert_open(file.path(), output) {
                    Ok(_) => {
                        println!("    succeeded in {:.3} s", start.elapsed().as_secs_f64())
                    }
                    Err(Error::ValidationFailed(problems)) => {
                        success = false;
                        println!("    FAILED");
                        for problem in problems {
                            println!("        {problem}");
                        }
                    }
                    Err(e) => {
                        success = false;
                        println!("    FAILED {e}");
                    }
                }
            }
        }
    }
    assert!(success);
}
