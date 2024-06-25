pub fn main() {
    write().unwrap();
    read().unwrap();
}

fn write() -> Result<(), omf::error::Error> {
    let mut writer =
        omf::file::Writer::new(std::fs::File::create("./examples/pyramid/pyramid.omf")?)?;
    // Pyramid surface
    let mut surface = omf::Element::new(
        "Pyramid surface",
        omf::Surface {
            origin: [0.0, 0.0, 0.0],
            vertices: writer.array_vertices(VERTICES.iter().copied())?,
            triangles: writer.array_triangles(TRIANGLES.iter().copied())?,
        },
    );
    surface.color = Some([255, 255, 0, 255]);
    // Pyramid outline
    let mut outline = omf::Element::new(
        "Pyramid outline",
        omf::LineSet {
            origin: [0.0, 0.0, 0.0],
            vertices: writer.array_vertices(VERTICES.iter().copied())?,
            segments: writer.array_segments(SEGMENTS.iter().copied())?,
        },
    );
    outline.color = Some([0, 0, 0, 255]);
    // Create and write the project
    let mut project = omf::Project {
        name: "Pyramid".to_owned(),
        description: "Contains a square pyramid.".to_owned(),
        author: "Somebody".to_owned(),
        application: "OMF Rust example".to_owned(),
        elements: vec![surface, outline],
        ..Default::default()
    };
    project.metadata.insert("revision".to_owned(), 1.2.into());
    project.metadata.insert(
        "tags".to_owned(),
        serde_json::Value::Array(vec!["foo".into(), "bar".into()]),
    );
    writer.finish(project)?;
    Ok(())
}

fn read() -> Result<(), omf::error::Error> {
    // Create the reader
    let mut reader =
        omf::file::Reader::new(std::fs::File::open("./examples/pyramid/pyramid.omf")?)?;
    reader.set_limits(omf::file::Limits {
        json_bytes: Some(16 * 1024),
        ..Default::default()
    });
    // Read the project
    let (project, problems) = reader.project()?;
    assert!(problems.is_empty(), "{problems:?}");
    println!("Project: {}", project.name);
    println!("Metadata:");
    for (key, value) in &project.metadata {
        println!("  {} = {:?}", key, value);
    }
    // Print the elements
    for element in &project.elements {
        println!("Element: \"{}\"", element.name);
        println!("  Description: {}", element.description);
        if let Some(color) = element.color {
            println!("  Color: {:?}", color);
        }
        match &element.geometry {
            omf::Geometry::Surface(surface) => {
                let vertices = reader
                    .array_vertices(&surface.vertices)?
                    .collect::<Result<Vec<_>, _>>()?;
                let triangles = reader
                    .array_triangles(&surface.triangles)?
                    .collect::<Result<Vec<_>, _>>()?;
                println!("  Surface:");
                println!("    Origin: {:?}", surface.origin);
                println!("    Vertices: {vertices:?}");
                println!("    Triangles: {triangles:?}");
            }
            omf::Geometry::LineSet(lines) => {
                let vertices = reader
                    .array_vertices(&lines.vertices)?
                    .collect::<Result<Vec<_>, _>>()?;
                let segments = reader
                    .array_segments(&lines.segments)?
                    .collect::<Result<Vec<_>, _>>()?;
                println!("  LineSet:");
                println!("    Origin: {:?}", lines.origin);
                println!("    Vertices: {vertices:?}");
                println!("    Segments: {segments:?}");
            }
            _ => println!("unexpected geometry"),
        }
    }
    Ok(())
}

const VERTICES: &[[f32; 3]] = &[
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

const TRIANGLES: &[[u32; 3]] = &[
    [0, 1, 4],
    [1, 2, 4],
    [2, 3, 4],
    [3, 0, 4],
    [0, 2, 1],
    [0, 3, 2],
];

const SEGMENTS: &[[u32; 2]] = &[
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [0, 4],
    [1, 4],
    [2, 4],
    [3, 4],
];
