use chrono::{DateTime, NaiveDate};
use flate2::{Compression, GzBuilder, read::GzDecoder};
use omf::{file::Writer, *};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions, remove_file},
    io::{Read, Write},
    path::Path,
    str::FromStr,
};
use zip::{ZipWriter, read::ZipArchive, write::SimpleFileOptions};

fn continuous_colormap() {
    let mut writer = Writer::new(create_file("tests/data/continuous_colormap.omf", b"")).unwrap();

    let mut project = Project::new("Continuous Colormap Test");
    project.description =
        "A simple OMF 2.0 project with continuous colormap attributes and different number ranges"
            .to_owned();

    let mut surface = Element::new(
        "Test Surface",
        Surface::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
            writer.array_triangles([[0, 1, 2], [0, 2, 3]]).unwrap(),
        ),
    );
    surface
        .attributes
        .push(Attribute::from_numbers_continuous_colormap(
            "Elevation",
            Location::Vertices,
            writer
                .array_numbers([Some(0.0_f64), Some(1.0), Some(2.0), Some(1.5)])
                .unwrap(),
            (0.0, 2.0),
            writer
                .array_gradient([[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]])
                .unwrap(),
        ));

    surface
        .attributes
        .push(Attribute::from_numbers_continuous_colormap(
            "DateRange",
            Location::Vertices,
            writer
                .array_numbers(
                    [
                        NaiveDate::from_ymd_opt(1995, 5, 1).unwrap(),
                        NaiveDate::from_ymd_opt(1996, 6, 1).unwrap(),
                        NaiveDate::from_ymd_opt(1997, 7, 1).unwrap(),
                        NaiveDate::from_ymd_opt(1998, 8, 1).unwrap(),
                    ]
                    .map(Some),
                )
                .unwrap(),
            (
                NaiveDate::from_ymd_opt(1995, 5, 1).unwrap(),
                NaiveDate::from_ymd_opt(1998, 8, 1).unwrap(),
            ),
            writer
                .array_gradient([[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]])
                .unwrap(),
        ));

    surface
        .attributes
        .push(Attribute::from_numbers_continuous_colormap(
            "DateTimeRange",
            Location::Vertices,
            writer
                .array_numbers(
                    [
                        DateTime::from_str("1995-05-01T05:01:00Z").unwrap(),
                        DateTime::from_str("1996-06-01T06:01:00Z").unwrap(),
                        DateTime::from_str("1997-07-01T07:01:00Z").unwrap(),
                        DateTime::from_str("1998-08-01T08:01:00Z").unwrap(),
                    ]
                    .map(Some),
                )
                .unwrap(),
            (
                DateTime::from_str("1995-05-01T05:01:00Z").unwrap(),
                DateTime::from_str("1998-08-01T08:01:00Z").unwrap(),
            ),
            writer
                .array_gradient([[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]])
                .unwrap(),
        ));

    surface
        .attributes
        .push(Attribute::from_numbers_continuous_colormap(
            "Elevation (f32)",
            Location::Vertices,
            writer
                .array_numbers([Some(0.0_f32), Some(1.0), Some(2.0), Some(1.5)])
                .unwrap(),
            (0.0, 2.0),
            writer
                .array_gradient([[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]])
                .unwrap(),
        ));

    surface
        .attributes
        .push(Attribute::from_numbers_continuous_colormap(
            "Elevation (i64)",
            Location::Vertices,
            writer
                .array_numbers([Some(0_i64), Some(100), Some(200), Some(150)])
                .unwrap(),
            (0, 200),
            writer
                .array_gradient([[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]])
                .unwrap(),
        ));

    project.elements.push(surface);

    let (.., warnings) = writer.finish(project).unwrap();
    assert!(warnings.is_empty());
}

fn duplicate_element_name() {
    let mut writer =
        Writer::new(create_file("tests/data/duplicate_element_name.omf", b"")).unwrap();

    let mut project = Project::new("Duplicate Element Name Test");
    project.description = "An OMF 2.0 project with a duplicate element name".to_owned();

    let element = Element::new(
        "Duplicate",
        PointSet::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
        ),
    );

    project.elements.push(element.clone());
    project.elements.push(element);

    writer.finish(project).unwrap();
}

fn missing_parquet() {
    let temp_file_path = "tests/data/missing_parquet.omf.tmp";
    let mut writer = Writer::new(create_file(temp_file_path, b"")).unwrap();

    let mut project = Project::new("Missing Parquet Test");
    project.description = "An OMF 2.0 project missing an expected parquet file".to_owned();

    let element = Element::new(
        "Missing",
        PointSet::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
        ),
    );

    project.elements.push(element);
    writer.finish(project).unwrap();

    // Replace archive with one only containing the index.json.gz file
    let mut new_zip = ZipWriter::new(create_file("tests/data/missing_parquet.omf", b""));
    add_file_from_another_zip(&mut new_zip, temp_file_path, "index.json.gz");

    let comment = format!("{FORMAT_NAME} {FORMAT_VERSION_MAJOR}.{FORMAT_VERSION_MINOR}");
    new_zip.set_comment(comment);

    new_zip.finish().unwrap();

    remove_file(temp_file_path).unwrap();
}

fn array_length_mismatch() {
    let temp_file_path = "tests/data/array_length_mismatch.omf.tmp";
    let mut writer = Writer::new(create_file(temp_file_path, b"")).unwrap();

    let mut project = Project::new("Array Length Mismatch Test");
    project.description = "An OMF 2.0 project missing an expected parquet file".to_owned();

    let element = Element::new(
        "Length Mismatch",
        PointSet::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
        ),
    );

    project.elements.push(element);
    writer.finish(project).unwrap();

    // Open and decompress the generated index.json file
    let file = File::open(temp_file_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let index_file = archive.by_name("index.json.gz").unwrap();

    let mut gz_dec = GzDecoder::new(index_file);

    let mut json_str = String::new();
    gz_dec.read_to_string(&mut json_str).unwrap();

    let mut index: Value = serde_json::from_str(&json_str).unwrap();

    // Set the vertex item count to an invalid value
    index["elements"][0]["geometry"]["vertices"]
        .as_object_mut()
        .unwrap()
        .insert("item_count".to_owned(), json!(999));

    let updated_index_json_str = serde_json::to_string(&index).unwrap();
    let mut update_index_json_gz_data = Vec::new();
    let mut gz = GzBuilder::new()
        .filename("index.json")
        .write(&mut update_index_json_gz_data, Compression::default());
    gz.write_all(updated_index_json_str.as_bytes()).unwrap();
    gz.finish().unwrap();

    // Create a new omf file with the modified index.json.gz and the parquet file
    let mut new_zip = ZipWriter::new(create_file("tests/data/array_length_mismatch.omf", b""));

    let comment = format!("{FORMAT_NAME} {FORMAT_VERSION_MAJOR}.{FORMAT_VERSION_MINOR}");
    new_zip.set_comment(comment);

    new_zip
        .start_file("index.json.gz", SimpleFileOptions::default())
        .unwrap();
    new_zip.write_all(&update_index_json_gz_data).unwrap();

    add_file_from_another_zip(&mut new_zip, temp_file_path, "1.parquet");
    new_zip.finish().unwrap();

    remove_file(temp_file_path).unwrap();
}

fn element_metadata() {
    let mut writer = Writer::new(create_file("tests/data/element_metadata.omf", b"")).unwrap();

    let mut project = Project::new("Element Metadata Test");
    project.description = "An OMF 2.0 project with element metadata".to_owned();

    let mut element = Element::new(
        "Element",
        PointSet::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
        ),
    );
    element
        .metadata
        .insert("sub-type".to_owned(), Value::String("point".to_owned()));
    element.metadata.insert(
        "date_created".to_owned(),
        Value::String("2024-10-14T00:00:00Z".to_owned()),
    );
    element.metadata.insert(
        "date_modified".to_owned(),
        Value::String("2024-10-15T00:00:00Z".to_owned()),
    );

    project.elements.push(element.clone());

    writer.finish(project).unwrap();
}

fn null_attribute_values() {
    let mut writer = Writer::new(create_file("tests/data/null_attribute_values.omf", b"")).unwrap();

    let mut project = Project::new("Null Attribute Values Test");
    project.description =
        "A simple OMF 2.0 project with attributes, some of which are null".to_owned();

    let mut surface = Element::new(
        "Test Surface",
        Surface::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
            writer.array_triangles([[0, 1, 2], [0, 2, 3]]).unwrap(),
        ),
    );

    surface.attributes.push(Attribute::from_numbers(
        "Numbers (f32)",
        Location::Vertices,
        writer
            .array_numbers([Some(0.0_f32), Some(1.0), None, Some(1.5)])
            .unwrap(),
    ));

    surface.attributes.push(Attribute::from_numbers(
        "Numbers (f64)",
        Location::Vertices,
        writer
            .array_numbers([Some(0.0_f64), Some(1.0), None, Some(1.5)])
            .unwrap(),
    ));

    surface.attributes.push(Attribute::from_numbers(
        "Numbers (i64)",
        Location::Vertices,
        writer
            .array_numbers([Some(0_i64), Some(100), None, Some(150)])
            .unwrap(),
    ));

    surface.attributes.push(Attribute::from_numbers(
        "Numbers (Date)",
        Location::Vertices,
        writer
            .array_numbers([
                Some(NaiveDate::from_ymd_opt(1995, 5, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(1996, 6, 1).unwrap()),
                None,
                Some(NaiveDate::from_ymd_opt(1998, 8, 1).unwrap()),
            ])
            .unwrap(),
    ));

    surface.attributes.push(Attribute::from_numbers(
        "Numbers (DateTime)",
        Location::Vertices,
        writer
            .array_numbers([
                Some(DateTime::from_str("1995-05-01T05:01:00Z").unwrap()),
                Some(DateTime::from_str("1996-06-01T06:01:00Z").unwrap()),
                None,
                Some(DateTime::from_str("1998-08-01T08:01:00Z").unwrap()),
            ])
            .unwrap(),
    ));

    surface.attributes.push(Attribute::from_categories(
        "Categories",
        Location::Vertices,
        writer
            .array_indices([Some(0), Some(1), None, Some(2)])
            .unwrap(),
        writer
            .array_names(["Zero".to_owned(), "One".to_owned(), "Two".to_owned()])
            .unwrap(),
        Some(
            writer
                .array_gradient([[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]])
                .unwrap(),
        ),
        [],
    ));

    surface.attributes.push(Attribute::from_booleans(
        "Booleans",
        Location::Vertices,
        writer
            .array_booleans([Some(false), Some(true), None, Some(false)])
            .unwrap(),
    ));

    surface.attributes.push(Attribute::from_colors(
        "Colors",
        Location::Vertices,
        writer
            .array_colors([
                Some([0, 0, 255, 255]),
                Some([0, 255, 0, 255]),
                None,
                Some([255, 0, 0, 255]),
            ])
            .unwrap(),
    ));

    project.elements.push(surface);

    let (.., warnings) = writer.finish(project).unwrap();
    assert!(warnings.is_empty());
}

fn add_file_from_another_zip(zip: &mut ZipWriter<File>, zip_file_name: &str, zip_file_entry: &str) {
    let file = File::open(zip_file_name).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let zip_entry = archive.by_name(zip_file_entry).unwrap();
    zip.raw_copy_file(zip_entry).unwrap();
}

fn create_file(name: &str, contents: &[u8]) -> File {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let mut f = OpenOptions::new()
        .truncate(true)
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    f.write_all(contents).unwrap();
    f
}

fn main() {
    continuous_colormap();
    duplicate_element_name();
    missing_parquet();
    array_length_mismatch();
    element_metadata();
    null_attribute_values();
}
