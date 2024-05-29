#![allow(dead_code, unused_imports)]

use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Read, Seek, Write},
    path::Path,
    str::FromStr,
};

use chrono::DateTime;
#[cfg(feature = "image")]
use image;

#[cfg(feature = "parquet")]
use omf::data::*;
use omf::{
    file::{Reader, Writer},
    validate::Validate,
    *,
};

const PYRAMID_VERTICES: [[f32; 3]; 5] = [
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
const PYRAMID_TRIANGLES: [[u32; 3]; 6] = [
    [0, 1, 4],
    [1, 2, 4],
    [2, 3, 4],
    [3, 0, 4],
    [0, 2, 1],
    [0, 3, 2],
];
const PYRAMID_SEGMENTS: [[u32; 2]; 8] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [0, 4],
    [1, 4],
    [2, 4],
    [3, 4],
];
const CUBE_VERTICES: [[f64; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];
const CUBE_TRIANGLES: [[u32; 3]; 12] = [
    [0, 2, 1],
    [0, 3, 2],
    [0, 1, 5],
    [0, 5, 4],
    [1, 2, 6],
    [1, 6, 5],
    [2, 3, 7],
    [2, 7, 6],
    [3, 0, 4],
    [3, 4, 7],
    [4, 5, 6],
    [4, 6, 7],
];
const CUBE_SEGMENTS: [[u32; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
];
const OCTREE_SUBBLOCKS: [([u32; 3], [u32; 6]); 11] = [
    ([0, 0, 0], [0, 0, 0, 4, 4, 4]),
    ([1, 0, 0], [0, 0, 0, 4, 4, 4]),
    ([0, 1, 0], [0, 0, 0, 4, 4, 4]),
    ([1, 1, 0], [0, 0, 0, 4, 4, 4]),
    ([0, 0, 1], [0, 0, 0, 4, 4, 4]),
    ([1, 0, 1], [0, 0, 0, 4, 4, 4]),
    ([0, 1, 1], [0, 0, 0, 4, 4, 4]),
    ([1, 1, 1], [0, 0, 0, 2, 2, 2]),
    ([1, 1, 1], [0, 2, 0, 1, 3, 1]),
    ([1, 1, 1], [2, 0, 0, 3, 1, 1]),
    ([1, 1, 1], [0, 0, 2, 1, 1, 3]),
];
const FREEFORM_SUBBLOCKS: [([u32; 3], [f32; 6]); 10] = [
    ([0, 0, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([1, 0, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([0, 1, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([1, 1, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([0, 0, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([1, 0, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([0, 1, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
    ([1, 1, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 0.3333]),
    ([1, 1, 1], [0.0, 0.0, 0.3333, 0.75, 0.75, 0.6666]),
    ([1, 1, 1], [0.0, 0.0, 0.6666, 0.5, 0.5, 1.0]),
];
const IMAGE_BYTES: &[u8] = include_bytes!("test.png");

fn temp_file(name: &str, contents: &[u8]) -> File {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
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

fn file_contents(mut file: File) -> Vec<u8> {
    file.rewind().unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

/// Creates an OMF file containing approximately one of everything.
///
///   project
//       metadata
//       surface (pyramid)
//           metadata
//           per-vertex numbers
//           per-primitive colors
//           per-vertex date-time
//       point-set (pyramid vertices)
//           per-vertex categories
//               metadata
//               integer sub-attribute
//           per-vertex vector2
//           per-vertex vector3
//       line-set (pyramid edges)
//           per-primitive text
//       grid surface (2x2)
//       block-model (2x2x2)
//           per-block filter
//       tensor block-model (2x2x2)
//       block-model regular sub-blocks (2x2x2 with 4x4x4 octree sub-blocks in one parent)
//       block-model free-form sub-blocks (2x2x2 with one parent split)
//       composite
//           surface (cube faces)
//           line-set (cube outline)
//       surface (rectangle)
//           projected texture
//           mapped texture
#[cfg(feature = "parquet")]
fn one_of_everything() -> (Vec<u8>, Project) {
    let mut writer = Writer::new(temp_file("one_of_everything.omf", b"")).unwrap();
    // Project.
    let mut project = Project::new("One of everything");
    project.description =
        "An OMF 2.0 project containing roughly one of every different type.".to_owned();
    project.author = "Tim Evans".to_owned();
    project.date = chrono::DateTime::from_str("1970-01-01T00:00:00Z").unwrap();
    project.metadata.insert("null".to_owned(), ().into());
    project.metadata.insert("bool".to_owned(), true.into());
    project.metadata.insert("string".to_owned(), "value".into());
    project.metadata.insert("number".to_owned(), 42.into());
    project
        .metadata
        .insert("array".to_owned(), vec![1, 2, 3].into());
    let mut obj: serde_json::Map<String, serde_json::Value> = Default::default();
    obj.insert("a".to_owned(), 1.into());
    obj.insert("b".to_owned(), 2.into());
    project.metadata.insert("object".to_owned(), obj.into());
    // Surface element.
    let mut surface = Element::new(
        "Pyramid surface",
        Surface::new(
            writer
                .array_vertices(PYRAMID_VERTICES.iter().copied())
                .unwrap(),
            writer
                .array_triangles(PYRAMID_TRIANGLES.iter().copied())
                .unwrap(),
        ),
    );
    surface.description = "A surface forming a pyramid".to_owned();
    surface.color = Some([255, 128, 0, 255]);
    surface.attributes.push(Attribute::from_numbers(
        "Numbers",
        Location::Vertices,
        writer
            .array_numbers([Some(1.0), Some(2.0), Some(3.0), Some(4.0), None])
            .unwrap(),
    ));
    surface.attributes.push(Attribute::from_colors(
        "Colors",
        Location::Primitives,
        writer
            .array_colors(
                [
                    [255_u8, 0, 0, 255],
                    [255, 255, 0, 255],
                    [0, 255, 0, 255],
                    [0, 0, 255, 255],
                    [255, 255, 255, 255],
                    [255, 255, 255, 255],
                ]
                .into_iter()
                .map(Some),
            )
            .unwrap(),
    ));
    surface
        .attributes
        .push(Attribute::from_numbers_discrete_colormap(
            "Date-times",
            Location::Vertices,
            writer
                .array_numbers(
                    [
                        DateTime::from_str("2000-01-01T00:00:00Z").unwrap(),
                        DateTime::from_str("2000-01-01T01:00:00Z").unwrap(),
                        DateTime::from_str("2000-01-01T02:00:00Z").unwrap(),
                        DateTime::from_str("2000-01-01T03:00:00Z").unwrap(),
                        DateTime::from_str("2000-01-01T04:00:00Z").unwrap(),
                    ]
                    .map(Some),
                )
                .unwrap(),
            writer
                .array_boundaries([
                    Boundary::Less(DateTime::from_str("2000-01-01T01:00:00Z").unwrap()),
                    Boundary::Less(DateTime::from_str("2000-01-01T02:00:00Z").unwrap()),
                    Boundary::LessEqual(DateTime::from_str("2000-01-01T03:00:00Z").unwrap()),
                ])
                .unwrap(),
            writer
                .array_gradient([
                    [0, 0, 255, 255],
                    [0, 255, 0, 255],
                    [255, 0, 0, 255],
                    [255, 255, 255, 255],
                ])
                .unwrap(),
        ));
    project.elements.push(surface);
    // PointSet element.
    let mut points = Element::new(
        "Pyramid points",
        PointSet::new(writer.array_vertices(PYRAMID_VERTICES.clone()).unwrap()),
    );
    let mut categories = Attribute::from_categories(
        "Categories",
        Location::Vertices,
        writer.array_indices([0_u32, 0, 0, 0, 1].map(Some)).unwrap(),
        writer
            .array_names(["Base", "Top"].map(|s| s.to_owned()))
            .unwrap(),
        Some(
            writer
                .array_gradient([[255_u8, 128, 0, 255], [0, 128, 255, 255]])
                .unwrap(),
        ),
        [Attribute::from_numbers(
            "Layer",
            Location::Categories,
            writer.array_numbers([Some(1_i64), Some(2)]).unwrap(),
        )],
    );
    categories.description = "Divides the points into top and base.".to_owned();
    categories.units = "whatever".to_owned();
    categories.metadata.insert("key".to_owned(), "value".into());
    points.attributes.push(categories);
    points.attributes.push(Attribute::from_vectors(
        "2D Vectors",
        Location::Vertices,
        writer
            .array_vectors([
                Some([1.0_f32, 0.0]),
                Some([1.0, 1.0]),
                Some([0.0, 1.0]),
                Some([0.0, 0.0]),
                None,
            ])
            .unwrap(),
    ));
    points.attributes.push(Attribute::from_vectors(
        "3D Vectors",
        Location::Vertices,
        writer
            .array_vectors([None, None, None, None, Some([0.0_f32, 0.0, 1.0])])
            .unwrap(),
    ));
    project.elements.push(points);
    // LineSet element.
    let mut line_set = Element::new(
        "Pyramid lines",
        LineSet::new(
            writer
                .array_vertices(PYRAMID_VERTICES.iter().copied())
                .unwrap(),
            writer
                .array_segments(PYRAMID_SEGMENTS.iter().copied())
                .unwrap(),
        ),
    );
    line_set.attributes.push(Attribute::from_strings(
        "Strings",
        Location::Primitives,
        writer
            .array_text([
                None,
                None,
                None,
                None,
                Some("sw".to_owned()),
                Some("se".to_owned()),
                Some("ne".to_owned()),
                Some("nw".to_owned()),
            ])
            .unwrap(),
    ));
    project.elements.push(line_set);
    // GridSurface element.
    project.elements.push(Element::new(
        "Pyramid grid surface",
        GridSurface::new(
            Orient2 {
                origin: [-1.5, -1.5, 0.0],
                u: [1.0, 0.0, 0.0],
                v: [0.0, 1.0, 0.0],
            },
            Grid2::Tensor {
                u: writer.array_scalars([1.0, 2.0]).unwrap(),
                v: writer.array_scalars([1.5, 1.5]).unwrap(),
            },
            Some(
                writer
                    .array_scalars([0.0_f32, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0])
                    .unwrap(),
            ),
        ),
    ));
    // Regular block model element.
    let mut block_model = Element::new(
        "Regular block model",
        BlockModel::new(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Regular {
                size: [1.0, 1.0, 1.0],
                count: [2, 2, 2],
            },
        ),
    );
    block_model.attributes.push(Attribute::from_booleans(
        "Filter",
        Location::Primitives,
        writer
            .array_booleans([false, false, false, false, false, false, false, true].map(Some))
            .unwrap()
            .into(),
    ));
    project.elements.push(block_model);
    // Tensor block model element.
    project.elements.push(Element::new(
        "Tensor block model",
        BlockModel::new(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Tensor {
                u: writer.array_scalars([0.6666, 1.333]).unwrap(),
                v: writer.array_scalars([0.6666, 1.333]).unwrap(),
                w: writer.array_scalars([1.0, 1.0]).unwrap(),
            },
        ),
    ));
    // Block model element, with regular sub-blocks.
    project.elements.push(Element::new(
        "Sub-blocked block model, regular",
        BlockModel::with_subblocks(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Regular {
                size: [1.0, 1.0, 1.0],
                count: [2, 2, 2],
            },
            Subblocks::Regular {
                count: [4, 4, 4],
                subblocks: writer.array_regular_subblocks(OCTREE_SUBBLOCKS).unwrap(),
                mode: Some(SubblockMode::Octree),
            },
        ),
    ));
    // Block model element, with free-form sub-blocks.
    project.elements.push(Element::new(
        "Sub-blocked block model, free-form",
        BlockModel::with_subblocks(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Regular {
                size: [1.0, 1.0, 1.0],
                count: [2, 2, 2],
            },
            Subblocks::Freeform {
                subblocks: writer.array_freeform_subblocks(FREEFORM_SUBBLOCKS).unwrap(),
            },
        ),
    ));
    // Composite elememnt.
    project.elements.push(Element::new(
        "Composite",
        Composite::new(vec![
            Element::new(
                "Cube faces",
                Surface::new(
                    writer.array_vertices(CUBE_VERTICES).unwrap(),
                    writer.array_triangles(CUBE_TRIANGLES).unwrap(),
                ),
            ),
            Element::new(
                "Cube edges",
                LineSet::new(
                    writer.array_vertices(CUBE_VERTICES).unwrap(),
                    writer.array_segments(CUBE_SEGMENTS).unwrap(),
                ),
            ),
        ]),
    ));
    // Projected texture.
    let mut rectangle = Element::new(
        "Textured",
        Surface::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
            writer.array_triangles([[0_u32, 1, 2], [0, 2, 3]]).unwrap(),
        ),
    );
    rectangle.attributes.push(Attribute::from_texture_project(
        "Projected",
        writer.image_bytes(IMAGE_BYTES).unwrap(),
        Orient2 {
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        },
        1.0,
        1.0,
    ));
    rectangle.attributes.push(Attribute::from_texture_map(
        "Mapped",
        writer.image_bytes(IMAGE_BYTES).unwrap(),
        Location::Vertices,
        writer
            .array_texcoords([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
            .unwrap(),
    ));
    project.elements.push(rectangle);
    // Done.
    let project_copy = project.clone();
    let (f, warnings) = writer.finish(project).unwrap();
    assert!(warnings.is_empty());
    (file_contents(f), project_copy)
}

#[cfg(feature = "parquet")]
fn one_of_everything_but_wrong() -> Project {
    let mut writer = file::Writer::new(temp_file("one_of_everything_but_wrong.omf", b"")).unwrap();
    // Project.
    let mut project = Project::new("One of everything but wrong");
    // Error: infinite.
    project.origin = [f64::INFINITY, 0.0, f64::NEG_INFINITY];
    // Surface element.
    let mut triangles = PYRAMID_TRIANGLES.clone();
    triangles[2][0] = 5; // Error: vertex index out of range.
    let mut surface = Element::new(
        "Pyramid surface",
        Surface::with_origin(
            writer.array_vertices(PYRAMID_VERTICES).unwrap(),
            writer.array_triangles(triangles).unwrap(),
            [f64::NAN, 0.0, 0.0], // Error: NaN in origin.
        ),
    );
    surface.attributes.push(Attribute::from_numbers(
        "Numbers",
        Location::Vertices,
        // Error: too many values.
        writer
            .array_numbers([
                Some(1.0_f64),
                Some(2.0),
                Some(3.0),
                Some(4.0),
                None,
                Some(5.0),
            ])
            .unwrap(),
    ));
    surface.attributes.push(Attribute::from_colors(
        "Numbers", // Warning: duplicate name.
        Location::Primitives,
        writer
            .array_colors([
                Some([255_u8, 0, 0, 255]),
                Some([255, 255, 0, 255]),
                Some([0, 255, 0, 255]),
                Some([0, 0, 255, 255]),
                Some([255, 255, 255, 255]),
                // Error: too few values.
            ])
            .unwrap(),
    ));
    project.elements.push(surface);
    // PointSet element.
    let mut points = Element::new(
        "Pyramid points",
        PointSet::new(writer.array_vertices(PYRAMID_VERTICES).unwrap()),
    );
    points.attributes.push(Attribute::from_categories(
        "Categories",
        Location::Vertices,
        // Error: category out of range.
        writer
            .array_indices([0_u32, 0, 0, 10, 1].map(Some))
            .unwrap()
            .into(),
        writer
            .array_names(["Base".to_owned(), "Top".to_owned()])
            .unwrap(),
        Some(
            writer
                .array_gradient([[255_u8, 128, 0, 255], [0, 128, 255, 255]])
                .unwrap(),
        ),
        [Attribute::from_numbers(
            "Layer",
            Location::Categories,
            writer
                .array_numbers([Some(1_i64), Some(2), Some(3)])
                .unwrap(), // Error: array too long
        )],
    ));
    points.attributes.push(Attribute::from_vectors(
        "2D Vectors",
        Location::Primitives, // Error: invalid location.
        writer
            .array_vectors([
                Some([1.0_f32, 0.0]),
                Some([1.0, 1.0]),
                Some([0.0, 1.0]),
                Some([0.0, 0.0]),
                None,
            ])
            .unwrap(),
    ));
    points.attributes.push(Attribute::from_vectors(
        "3D Vectors",
        Location::Vertices,
        // Error: too many values.
        writer
            .array_vectors([Some([0.0_f32, 0.0, 1.0]), None, None, None, None, None])
            .unwrap(),
    ));
    project.elements.push(points);
    // LineSet element.
    let mut segments = PYRAMID_SEGMENTS.clone();
    segments[0][0] = 10; // Error: vertex index out of range.
    let mut line_set = Element::new(
        "Pyramid lines",
        LineSet::new(
            writer.array_vertices(PYRAMID_VERTICES).unwrap(),
            writer.array_segments(segments).unwrap(),
        ),
    );
    line_set.attributes.push(Attribute::from_strings(
        "Strings",
        Location::Primitives,
        writer
            .array_text([
                None,
                None,
                None,
                None,
                Some("sw".to_owned()),
                Some("se".to_owned()),
                Some("ne".to_owned()),
            ])
            .unwrap(),
    ));
    project.elements.push(line_set);
    // GridSurface element.
    project.elements.push(Element::new(
        "Pyramid grid surface",
        GridSurface::new(
            Orient2 {
                origin: [-1.5, -1.5, 0.0],
                // Error: not unit vector.
                u: [0.8, 0.0, 0.0],
                // Error: not orthogonal.
                v: [0.2, 1.0, 0.0],
            },
            Grid2::Tensor {
                u: writer.array_scalars([1.0, 2.0]).unwrap(),
                v: writer.array_scalars([1.5, 1.5]).unwrap(),
            },
            // Error: wrong length.
            Some(
                writer
                    .array_scalars([0.0_f32, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0])
                    .unwrap(),
            ),
        ),
    ));
    // Regular block model element.
    let mut block_model = Element::new(
        "Regular block model",
        BlockModel::new(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Regular {
                size: [1.0, 0.0, 1.0], // Error: zero size.
                count: [2, 2, 0],      // Error: zero count.
            },
        ),
    );
    block_model.attributes.push(Attribute::from_booleans(
        "Filter",
        Location::Primitives,
        writer
            .array_booleans(
                // Error: too few values.
                [false, false, false, false, false, false, false, true].map(Some),
            )
            .unwrap(),
    ));
    project.elements.push(block_model);
    // Tensor block model element.
    project.elements.push(Element::new(
        "Tensor block model",
        BlockModel::new(
            Orient3::from_origin([-1.0, -1.0, f64::NAN]), // Error: Nan origin
            Grid3::Tensor {
                u: writer.array_scalars([0.6666, 1.333]).unwrap(),
                v: writer.array_scalars([0.6666, 1.333]).unwrap(),
                w: writer.array_scalars([1.0, 0.0]).unwrap(), // Error: zero size
            },
        ),
    ));
    // Block model element, with regular sub-blocks.
    let mut subblocks = OCTREE_SUBBLOCKS.clone();
    subblocks[4].0[2] = 2; // Error: parent index out of range
    subblocks[2].1[5] = 10; // Error: corner out of range
    project.elements.push(Element::new(
        "Sub-blocked block model, regular",
        BlockModel::with_subblocks(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Regular {
                size: [1.0, 1.0, 1.0],
                count: [2, 2, 2],
            },
            Subblocks::Regular {
                count: [4, 4, 4],
                subblocks: writer.array_regular_subblocks(subblocks).unwrap(),
                mode: Some(SubblockMode::Octree),
            },
        ),
    ));
    // Block model element, with free-form sub-blocks.
    project.elements.push(Element::new(
        "Sub-blocked block model, free-form",
        BlockModel::with_subblocks(
            Orient3::from_origin([-1.0, -1.0, -1.0]),
            Grid3::Regular {
                size: [1.0, 1.0, 1.0],
                count: [2, 2, 2],
            },
            Subblocks::Freeform {
                subblocks: writer.array_freeform_subblocks(FREEFORM_SUBBLOCKS).unwrap(),
            },
        ),
    ));
    // Composite elememnt.
    project.elements.push(Element::new(
        "Composite",
        Composite::new(vec![
            Element::new(
                "Cube faces",
                Surface::new(
                    writer.array_vertices(CUBE_VERTICES).unwrap(),
                    writer.array_triangles(CUBE_TRIANGLES).unwrap(),
                ),
            ),
            Element::new(
                "Cube faces", // Warning: duplicate name.
                LineSet::new(
                    writer.array_vertices(CUBE_VERTICES).unwrap(),
                    writer.array_segments(CUBE_SEGMENTS).unwrap(),
                ),
            ),
        ]),
    ));
    // Projected texture.
    let mut rectangle = Element::new(
        "Textured",
        Surface::new(
            writer
                .array_vertices([
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ])
                .unwrap(),
            writer.array_triangles([[0_u32, 1, 2], [0, 2, 3]]).unwrap(),
        ),
    );
    rectangle.attributes.push(Attribute::from_texture_project(
        "Projected",
        writer.image_bytes(IMAGE_BYTES).unwrap(),
        Orient2 {
            origin: [0.0, f64::INFINITY, 0.0], // Error: infinite.
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        },
        1.0,
        1.0,
    ));
    rectangle.attributes.push(Attribute::from_texture_map(
        "Mapped",
        writer.image_bytes(IMAGE_BYTES).unwrap(),
        Location::Vertices,
        // Error: too many values.
        writer
            .array_texcoords([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [2.0, 2.0]])
            .unwrap(),
    ));
    project.elements.push(rectangle);
    // Done.
    project
}

fn check_loc_lens(element: &Element, required: &[(Location, u64)]) {
    for loc in [
        Location::Vertices,
        Location::Primitives,
        Location::Primitives,
        Location::Subblocks,
        Location::Elements,
    ] {
        let req = required
            .iter()
            .chain(Some(&(Location::Projected, 0)))
            .find_map(|(l, n)| if *l == loc { Some(*n) } else { None });
        let val = element.location_len(loc);
        if val != req {
            let name = format!("{:?}", &element.geometry)
                .split('(')
                .next()
                .unwrap()
                .to_owned();
            panic!("{name}::location_len({loc:?}) was {val:?}, should have been {req:?}",)
        }
    }
}

#[cfg(feature = "parquet")]
#[test]
fn location_lengths() {
    let (_, project) = one_of_everything();
    macro_rules! check {
        ($index:literal, $( $loc:ident = $n:literal ),*) => {
            check_loc_lens(
                &&project.elements[$index],
                &[ $( (Location::$loc, $n) ),* ],
            )
        };
    }
    check!(0, Vertices = 5, Primitives = 6); // surface
    check!(1, Vertices = 5); // point-set
    check!(2, Vertices = 5, Primitives = 8); // line-set
    check!(3, Vertices = 9, Primitives = 4); // grid-surface
    check!(4, Primitives = 8, Vertices = 27); // block-model
    check!(5, Primitives = 8, Vertices = 27); // tensor block-model
    check!(6, Primitives = 8, Vertices = 27, Subblocks = 11); // sub-blocks
    check!(7, Primitives = 8, Vertices = 27, Subblocks = 10); // free-form sub-blocks
    check!(8, Elements = 2); // composite
}

#[cfg(feature = "parquet")]
#[test]
fn load_one_of_everything() {
    let (_, project) = one_of_everything();
    let bytes = std::fs::read("tests/one_of_everything.omf.json").unwrap();
    let loaded_project: Project = serde_json::from_slice(&bytes).unwrap();
    assert!(
        loaded_project == project,
        "one_of_everything project differs from tests/one_of_everything.omf"
    );
}

#[cfg(feature = "parquet")]
#[ignore = "used to update benchmark"]
#[test]
fn update_one_of_everything() {
    let (bytes, project) = one_of_everything();
    std::fs::write("tests/one_of_everything.omf", bytes).unwrap();
    std::fs::write(
        "tests/one_of_everything.omf.json",
        serde_json::to_string_pretty(&project).unwrap().as_bytes(),
    )
    .unwrap();
}

#[cfg(feature = "parquet")]
#[test]
fn write_and_read_one_of_everything() {
    let (bytes, base_project) = one_of_everything();
    let f = temp_file("write_and_read_one_of_everything.omf", &bytes);
    let reader = Reader::new(f).unwrap();
    let (project, warnings) = reader.project().unwrap();
    let warning_strings: Vec<String> = warnings.into_iter().map(|p| p.to_string()).collect();
    assert_eq!(warning_strings, Vec::<String>::new());
    assert!(project == base_project, "read project differs");
    macro_rules! geometry {
        ($name:ident: $geom_type:ident = $source:ident[$index:literal]) => {
            let Element {
                geometry: Geometry::$geom_type($name),
                ..
            } = &$source.elements[$index]
            else {
                panic!("wrong geometry");
            };
        };
    }
    macro_rules! check_array {
        ($load:ident($array:expr): $enum:ident :: $var:ident, $expected:expr) => {
            let $enum::$var(iter) = reader.$load($array).unwrap() else {
                panic!("wrong type");
            };
            let data = iter.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(data, $expected);
        };
        ($load:ident($array:expr), $expected:expr) => {
            let iter = reader.$load($array).unwrap();
            let data = iter.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(data, $expected);
        };
    }
    macro_rules! check_attr {
        ($load:ident($e:literal, $a:literal, $attr:ident): $enum:ident :: $var:ident, $expected:expr) => {
            let AttributeData::$attr { values, .. } = &project.elements[$e].attributes[$a].data
            else {
                panic!("wrong attribute type");
            };
            check_array!($load(values): $enum :: $var, $expected);
        };
        ($load:ident($e:literal, $a:literal, $attr:ident), $expected:expr) => {
            let AttributeData::$attr { values, .. } = &project.elements[$e].attributes[$a].data
            else {
                panic!("wrong attribute type");
            };
            check_array!($load(values), $expected);
        };
    }
    // Surface element.
    geometry!(surface: Surface = project[0]);
    check_array!(array_vertices(&surface.vertices): Vertices::F32, PYRAMID_VERTICES);
    check_array!(array_triangles(&surface.triangles), PYRAMID_TRIANGLES);
    check_attr!(array_numbers(0, 0, Number): Numbers::F64, [
        Some(1.0), Some(2.0), Some(3.0), Some(4.0), None,
    ]);
    check_attr!(
        array_colors(0, 1, Color),
        [
            Some([255, 0, 0, 255]),
            Some([255, 255, 0, 255]),
            Some([0, 255, 0, 255]),
            Some([0, 0, 255, 255]),
            Some([255, 255, 255, 255]),
            Some([255, 255, 255, 255]),
        ]
    );
    let AttributeData::Number {
        values,
        colormap:
            Some(NumberColormap::Discrete {
                boundaries,
                gradient,
            }),
    } = &project.elements[0].attributes[2].data
    else {
        panic!("wrong type");
    };
    check_array!(array_numbers(values): Numbers::DateTime, [
        DateTime::from_str("2000-01-01T00:00:00Z").unwrap(),
        DateTime::from_str("2000-01-01T01:00:00Z").unwrap(),
        DateTime::from_str("2000-01-01T02:00:00Z").unwrap(),
        DateTime::from_str("2000-01-01T03:00:00Z").unwrap(),
        DateTime::from_str("2000-01-01T04:00:00Z").unwrap(),
    ].map(Some));
    check_array!(
        array_boundaries(boundaries): Boundaries::DateTime,
        [
            Boundary::Less(DateTime::from_str("2000-01-01T01:00:00Z").unwrap()),
            Boundary::Less(DateTime::from_str("2000-01-01T02:00:00Z").unwrap()),
            Boundary::LessEqual(DateTime::from_str("2000-01-01T03:00:00Z").unwrap()),
        ]
    );
    check_array!(
        array_gradient(gradient),
        [
            [0, 0, 255, 255],
            [0, 255, 0, 255],
            [255, 0, 0, 255],
            [255, 255, 255, 255],
        ]
    );
    // PointSet element.
    geometry!(point_set: PointSet = project[1]);
    check_array!(array_vertices(&point_set.vertices): Vertices::F32, PYRAMID_VERTICES);
    check_attr!(array_indices(1, 0, Category), [0, 0, 0, 0, 1].map(Some));
    check_attr!(array_vectors(1, 1, Vector): Vectors::F32x2, [
        Some([1.0, 0.0]), Some([1.0, 1.0]), Some([0.0, 1.0]), Some([0.0, 0.0]), None,
    ]);
    check_attr!(array_vectors(1, 2, Vector): Vectors::F32x3, [
        None, None, None, None, Some([0.0, 0.0, 1.0]),
    ]);
    let AttributeData::Category {
        names,
        gradient: Some(c),
        attributes,
        ..
    } = &project.elements[1].attributes[0].data
    else {
        panic!("wrong attr type");
    };
    check_array!(array_names(names), ["Base", "Top"]);
    check_array!(
        array_gradient(c),
        [[255_u8, 128, 0, 255], [0, 128, 255, 255]]
    );
    let AttributeData::Number {
        values,
        colormap: None,
    } = &attributes[0].data
    else {
        panic!("wrong attribute data type");
    };
    check_array!(array_numbers(values): Numbers::I64, [Some(1), Some(2)]);
    // LineSet element.
    geometry!(line_set: LineSet = project[2]);
    check_array!(array_vertices(&line_set.vertices): Vertices::F32, PYRAMID_VERTICES);
    check_array!(array_segments(&line_set.segments), PYRAMID_SEGMENTS);
    let AttributeData::Text { values } = &project.elements[2].attributes[0].data else {
        panic!("wrong attr type");
    };
    check_array!(
        array_text(values),
        [
            None,
            None,
            None,
            None,
            Some("sw".to_owned()),
            Some("se".to_owned()),
            Some("ne".to_owned()),
            Some("nw".to_owned()),
        ]
    );
    // GridSurface element.
    geometry!(grid_surface: GridSurface = project[3]);
    check_array!(
        array_scalars(grid_surface.heights.as_ref().unwrap()): Scalars::F32,
        [0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0]
    );
    let Grid2::Tensor { u, v } = &grid_surface.grid else {
        panic!("not tensor");
    };
    check_array!(array_scalars(u): Scalars::F64, [1.0, 2.0]);
    check_array!(array_scalars(v): Scalars::F64, [1.5, 1.5]);
    // Regular block model element.
    geometry!(_regular_bm: BlockModel = project[4]);
    check_attr!(
        array_booleans(4, 0, Boolean),
        [false, false, false, false, false, false, false, true].map(Some)
    );
    geometry!(tensor_bm: BlockModel = project[5]);
    let Grid3::Tensor { u, v, w } = &tensor_bm.grid else {
        panic!("not tensor");
    };
    check_array!(array_scalars(u): Scalars::F64, [0.6666, 1.333]);
    check_array!(array_scalars(v): Scalars::F64, [0.6666, 1.333]);
    check_array!(array_scalars(w): Scalars::F64, [1.0, 1.0]);
    // Block model element, with regular sub-blocks.
    geometry!(regular_subblocks: BlockModel = project[6]);
    let Some(Subblocks::Regular { subblocks, .. }) = &regular_subblocks.subblocks else {
        panic!("not regular sub-blocked");
    };
    check_array!(
        array_regular_subblocks(subblocks),
        [
            ([0, 0, 0], [0, 0, 0, 4, 4, 4]),
            ([1, 0, 0], [0, 0, 0, 4, 4, 4]),
            ([0, 1, 0], [0, 0, 0, 4, 4, 4]),
            ([1, 1, 0], [0, 0, 0, 4, 4, 4]),
            ([0, 0, 1], [0, 0, 0, 4, 4, 4]),
            ([1, 0, 1], [0, 0, 0, 4, 4, 4]),
            ([0, 1, 1], [0, 0, 0, 4, 4, 4]),
            ([1, 1, 1], [0, 0, 0, 2, 2, 2]),
            ([1, 1, 1], [0, 2, 0, 1, 3, 1]),
            ([1, 1, 1], [2, 0, 0, 3, 1, 1]),
            ([1, 1, 1], [0, 0, 2, 1, 1, 3]),
        ]
    );
    // Block model element, with free-form sub-blocks.
    geometry!(freeform_subblocks: BlockModel = project[7]);
    let Some(Subblocks::Freeform { subblocks, .. }) = &freeform_subblocks.subblocks else {
        panic!("not free-form sub-blocked");
    };
    check_array!(
        array_freeform_subblocks(subblocks): FreeformSubblocks::F32,
        [
            ([0, 0, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 0, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([0, 1, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 1, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([0, 0, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 0, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([0, 1, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 1, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 0.3333]),
            ([1, 1, 1], [0.0, 0.0, 0.3333, 0.75, 0.75, 0.6666]),
            ([1, 1, 1], [0.0, 0.0, 0.6666, 0.5, 0.5, 1.0]),
        ]
    );
    // Composite element.
    geometry!(comp: Composite = project[8]);
    geometry!(comp_surface: Surface = comp[0]);
    check_array!(array_vertices(&comp_surface.vertices): Vertices::F64, CUBE_VERTICES);
    check_array!(array_triangles(&comp_surface.triangles), CUBE_TRIANGLES);
    geometry!(comp_line_set: LineSet = comp[1]);
    check_array!(array_vertices(&comp_line_set.vertices): Vertices::F64, CUBE_VERTICES);
    check_array!(array_segments(&comp_line_set.segments), CUBE_SEGMENTS);
    // Textures.
    geometry!(_rectangle: Surface = project[9]);
    let AttributeData::ProjectedTexture { image, .. } = &project.elements[9].attributes[0].data
    else {
        panic!("wrong attr type");
    };
    let bytes = reader.array_bytes(image).unwrap();
    assert_eq!(bytes, IMAGE_BYTES);
    let AttributeData::MappedTexture {
        image, texcoords, ..
    } = &project.elements[9].attributes[1].data
    else {
        panic!("wrong attr type");
    };
    let bytes = reader.array_bytes(image).unwrap();
    assert_eq!(bytes, IMAGE_BYTES);
    check_array!(array_texcoords(texcoords): Texcoords::F64,
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    );
}

#[cfg(feature = "parquet")]
#[test]
fn validate_one_of_everything_but_wrong() {
    let mut project = one_of_everything_but_wrong();
    let problems = project.validate().unwrap_err();
    let mut expected = vec![
        "Error: 'Project::origin' must be finite, inside 'One of everything but wrong'",
        "Error: 'Surface::origin' must be finite, inside 'Pyramid surface'",
        "Warning: 'Element::attributes[..]::name' contains duplicate of \"Numbers\", inside 'Pyramid surface'",
        "Error: 'Attribute' length 6 does not match geometry (5), inside 'Numbers'",
        "Error: 'Attribute' length 5 does not match geometry (6), inside 'Numbers'",
        "Error: 'Attribute::location' is Primitives which is not valid on PointSet geometry, inside '2D Vectors'",
        "Error: 'Attribute' length 6 does not match geometry (5), inside '3D Vectors'",
        "Error: 'Attribute' length 7 does not match geometry (8), inside 'Strings'",
        "Error: 'Orient2::u' must be a unit vector but [0.8, 0.0, 0.0] length is 0.8, inside 'Pyramid grid surface'",
        "Error: 'Orient2::v' must be a unit vector but [0.2, 1.0, 0.0] length is 1.0198039, inside 'Pyramid grid surface'",
        "Error: 'Orient2' vectors are not orthogonal: [0.8, 0.0, 0.0] [0.2, 1.0, 0.0], inside 'Pyramid grid surface'",
        "Error: 'GridSurface::heights' length 8 does not match geometry (9), inside 'Pyramid grid surface'",
        "Error: 'Grid3::Regular::size' must be greater than zero, inside 'Regular block model'",
        "Error: 'Grid3::Regular::count' must be greater than zero, inside 'Regular block model'",
        "Error: 'Attribute' length 8 does not match geometry (0), inside 'Filter'",
        "Error: 'Orient3::origin' must be finite, inside 'Tensor block model'",
        "Warning: 'Composite::elements[..]::name' contains duplicate of \"Cube faces\", inside 'Composite'",
        "Error: 'Orient2::origin' must be finite, inside 'Projected'",
        "Error: 'Attribute' length 5 does not match geometry (4), inside 'Mapped'",
        "Error: 'Attribute' length 3 does not match geometry (2), inside 'Layer'",
        "Error: 'Surface::triangles' array contains invalid data: index value 5 exceeds the maximum index 4, inside 'Pyramid surface'",
        "Error: 'AttributeData::Category::values' array contains invalid data: index value 10 exceeds the maximum index 1, inside 'Categories'",
        "Error: 'LineSet::segments' array contains invalid data: index value 10 exceeds the maximum index 4, inside 'Pyramid lines'",
        "Error: 'Grid3::Tensor::w' array contains invalid data: size value 0 is zero or less, inside 'Tensor block model'",
        "Error: 'Subblocks::Regular::subblocks' array contains invalid data: block index [0, 0, 2] exceeds the maximum index [1, 1, 1], inside 'Sub-blocked block model, regular'",
        "Error: 'Subblocks::Regular::subblocks' array contains invalid data: sub-block [0, 0, 0] to [4, 4, 10] exceeds the maximum [4, 4, 4], inside 'Sub-blocked block model, regular'",
        "Error: 'Subblocks::Regular::subblocks' array contains invalid data: sub-block [0, 0, 0] to [4, 4, 10] is invalid for Octree mode, inside 'Sub-blocked block model, regular'",
        ];
    let mut unexpected = Vec::new();
    for p in problems.into_vec() {
        let s = p.to_string();
        if let Some(index) = expected.iter().position(|e| *e == &s) {
            expected.remove(index);
        } else {
            unexpected.push(p.to_string());
        }
    }
    if !unexpected.is_empty() || !expected.is_empty() {
        panic!("unexpected problems: {unexpected:#?}\nexpected but not found: {expected:#?}");
    }
}

#[cfg(all(feature = "image", feature = "parquet"))]
#[test]
fn write_and_read_images() {
    let points = [[1_f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
    let test_image = image::open("tests/test.png").unwrap();
    let image::DynamicImage::ImageRgb8(test_image_rgb8) = &test_image else {
        panic!("expected Rgb8");
    };
    let mut writer = file::Writer::new(temp_file("write_and_read_images.omf", b"")).unwrap();
    let mut project = Project::new("test");
    let arr = writer.array_vertices(points).unwrap();
    let jpeg = writer.image_jpeg(&test_image_rgb8, 85).unwrap();
    let png = writer.image_png(&test_image).unwrap();
    let mut points = Element::new("points", PointSet::new(arr));
    points.attributes.push(Attribute::from_texture_project(
        "jpeg texture",
        jpeg,
        Default::default(),
        10.0,
        10.0,
    ));
    points.attributes.push(Attribute::from_texture_project(
        "png texture",
        png,
        Default::default(),
        10.0,
        10.0,
    ));
    project.elements.push(points);
    let original = project.clone();
    let (mut file, warnings) = writer.finish(project).unwrap();
    assert!(warnings.is_empty());

    file.rewind().unwrap();
    let reader = Reader::new(file).unwrap();
    let (loaded, warnings) = reader.project().unwrap();
    assert!(warnings.is_empty());
    assert_eq!(loaded, original);
    let AttributeData::ProjectedTexture { image: i, .. } = &loaded.elements[0].attributes[0].data
    else {
        panic!("wrong project structure");
    };
    let AttributeData::ProjectedTexture { image: j, .. } = &loaded.elements[0].attributes[1].data
    else {
        panic!("wrong project structure");
    };
    let loaded_jpeg = reader.image(&i).unwrap();
    let loaded_png = reader.image(&j).unwrap();
    // jpeg will be different due to lossy compression
    assert!(loaded_jpeg != test_image);
    // png will be exactly the same
    assert!(loaded_png == test_image);
}
