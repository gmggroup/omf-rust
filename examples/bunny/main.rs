use std::{fs::File, io::BufRead};

fn iter_to_array<T: Default, const N: usize>(iterable: impl IntoIterator<Item = T>) -> [T; N] {
    let mut iter = iterable.into_iter();
    std::array::from_fn(|_| iter.next().unwrap())
}

fn main() {
    write();
    read();
}

/// Parses bunny.obj in Wavefront OBJ format and writes it to bunny.omf.
fn write() {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    for line in std::io::BufReader::new(File::open("./examples/bunny/bunny.obj").unwrap()).lines() {
        let line = line.unwrap();
        if line.starts_with('v') {
            vertices.push(iter_to_array(
                line.split_whitespace().skip(1).map(|s| s.parse().unwrap()),
            ));
        } else if line.starts_with('f') {
            triangles.push(iter_to_array(
                line.split_whitespace()
                    .skip(1)
                    .map(|s| s.parse::<u32>().unwrap() - 1),
            ));
        }
    }
    let mut writer =
        omf::file::Writer::new(File::create("./examples/bunny/bunny.omf").unwrap()).unwrap();
    let mut project = omf::Project::default();
    project.elements.push(omf::Element::new(
        "Bunny surface",
        omf::Surface {
            origin: [0.0, 0.0, 0.0],
            vertices: writer.array_vertices(vertices.iter().copied()).unwrap(),
            triangles: writer.array_triangles(triangles.iter().copied()).unwrap(),
        },
    ));
    "The Stanford bunny, with the holes patched, and rotated to make Z up."
        .clone_into(&mut project.description);
    project.metadata.insert(
        "source".to_owned(),
        "Stanford University Computer Graphics Laboratory".into(),
    );
    writer.finish(project).unwrap();
}

/// Reads bunny.omf back and prints some details.
fn read() {
    let reader = omf::file::Reader::new(File::open("./examples/bunny/bunny.omf").unwrap()).unwrap();
    let (project, _) = reader.project().unwrap();
    println!("description: {:?}", project.description);
    println!("source: {}", project.metadata.get("source").unwrap());
    let omf::Geometry::Surface(surface) = &project.elements[0].geometry else {
        panic!("wrong geometry type");
    };
    let vertices = reader.array_vertices(&surface.vertices).unwrap();
    let triangles = reader.array_triangles(&surface.triangles).unwrap();
    println!("First five vertices:");
    for [x, y, z] in vertices.take(5).map(Result::unwrap) {
        println!("  {x:.2} {y:.2} {z:.2}");
    }
    println!("First five triangles:");
    for [x, y, z] in triangles.take(5).map(Result::unwrap) {
        println!("  {x} {y} {z}");
    }
}
