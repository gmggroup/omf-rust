use chrono::{DateTime, NaiveDate};
use omf::{file::Writer, *};
use std::{
    fs::{remove_file, File, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
};
use zip::{read::ZipArchive, ZipWriter};

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

    project.elements.push(surface);

    let (.., warnings) = writer.finish(project).unwrap();
    assert!(warnings.is_empty());
}

fn duplicate_element_name() {
    let mut writer =
        Writer::new(create_file("tests/data/duplicate_element_name.omf", b"")).unwrap();

    let mut project = Project::new("Duplicate Element Name");
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
    project.elements.push(element.clone());

    writer.finish(project).unwrap();
}

fn missing_parquet() {
    let temp_file_path = "tests/data/missing_parquet.omf.tmp";
    let mut writer = Writer::new(create_file(temp_file_path, b"")).unwrap();

    let mut project = Project::new("Missing Parquet");
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
    let file = File::open(temp_file_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let index_file = archive.by_name("index.json.gz").unwrap();

    let mut new_zip = ZipWriter::new(create_file("tests/data/missing_parquet.omf", b""));

    let comment = format!("{FORMAT_NAME} {FORMAT_VERSION_MAJOR}.{FORMAT_VERSION_MINOR}");
    new_zip.set_comment(comment);

    new_zip.raw_copy_file(index_file).unwrap();
    new_zip.finish().unwrap();

    remove_file(temp_file_path).unwrap();
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
}
