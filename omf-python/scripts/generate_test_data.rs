use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
};

use chrono::{DateTime, NaiveDate};
use omf::{file::Writer, *};

fn continuous_colormap() -> () {
    let mut writer = Writer::new(temp_file("tests/data/continuous_colormap.omf", b"")).unwrap();

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

fn temp_file(name: &str, contents: &[u8]) -> File {
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

fn main() -> () {
    continuous_colormap();
}
