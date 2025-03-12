use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use glam::{DMat3, DVec3, UVec3};

fn main() {
    write();
    read();
}

fn write() {
    // Parse CSV block model.
    //
    // The data missing from the CSV file is:
    //  - Parent block size is 400 400 400.
    //  - Size in parent blocks is 8 6 8 = 384.
    //  - Minimum corner is -1662 -1340 -423.
    //  - The model is rotated by 10 degrees around the Z axis, centered on the minimum corner.
    let origin = DVec3::new(-1662.0, -1340.0, -423.0);
    let mat = DMat3::from_axis_angle(DVec3::new(0.0, 0.0, 1.0), 10.0_f64.to_radians());
    let mat_inv = mat.inverse();
    let mut parent_indices = Vec::new();
    let mut subblocks = Vec::new();
    let mut bunny = Vec::new();
    for line in BufReader::new(File::open("./examples/bunny_blocks/bunny_blocks.csv").unwrap())
        .lines()
        .skip(1)
    {
        let line = line.unwrap();
        if line.starts_with('#') {
            continue;
        }
        let mut parts = line.split(',').skip(1);
        let centroid = DVec3::new(
            parts.next().unwrap().parse().unwrap(),
            parts.next().unwrap().parse().unwrap(),
            parts.next().unwrap().parse().unwrap(),
        );
        let size = DVec3::new(
            parts.next().unwrap().parse::<f64>().unwrap(),
            parts.next().unwrap().parse::<f64>().unwrap(),
            parts.next().unwrap().parse::<f64>().unwrap(),
        );
        let category = parts.next().unwrap();
        let relative = mat_inv * (centroid - origin);
        let parent_index = (relative / 400.0).floor().as_uvec3();
        let min = ((relative - size / 2.0) / 100.0).round().as_uvec3() - parent_index * 4;
        let max = ((relative + size / 2.0) / 100.0).round().as_uvec3() - parent_index * 4;
        parent_indices.push(parent_index);
        subblocks.push((min, max));
        bunny.push(category == "Body");
    }
    // Write OMF block model.
    let mut writer =
        omf::file::Writer::new(File::create("./examples/bunny_blocks/bunny_blocks.omf").unwrap())
            .unwrap();
    let mut project = omf::Project::default();
    let mut element = omf::Element::new(
        "Octree Block Model",
        omf::BlockModel {
            orient: omf::Orient3 {
                origin: origin.into(),
                u: mat.x_axis.into(),
                v: mat.y_axis.into(),
                w: mat.z_axis.into(),
            },
            grid: omf::Grid3::Regular {
                size: [400.0, 400.0, 400.0],
                count: [8, 6, 8],
            },
            subblocks: Some(omf::Subblocks::Regular {
                subblocks: writer
                    .array_regular_subblocks(parent_indices.into_iter().zip(subblocks).map(
                        |(p, (min, max))| {
                            (p.to_array(), [min.x, min.y, min.z, max.x, max.y, max.z])
                        },
                    ))
                    .unwrap(),
                count: [4, 4, 4],
                mode: Some(omf::SubblockMode::Octree),
            }),
        },
    );
    element.attributes.push(omf::Attribute::from_categories(
        "Bunny",
        omf::Location::Subblocks,
        writer
            .array_indices(bunny.into_iter().map(|b| Some(b as u32)))
            .unwrap(),
        writer
            .array_names(["Air".to_owned(), "Body".to_owned()])
            .unwrap(),
        Some(
            writer
                .array_gradient([[173, 216, 230, 255], [232, 193, 138, 255]])
                .unwrap(),
        ),
        [],
    ));
    project.elements.push(element);
    "The Stanford bunny, as an octree block model.".clone_into(&mut project.description);
    writer.finish(project).unwrap();
}

fn read() {
    let reader =
        omf::file::Reader::new(File::open("./examples/bunny_blocks/bunny_blocks.omf").unwrap())
            .unwrap();
    let (project, _) = reader.project().unwrap();
    let omf::Geometry::BlockModel(block_model) = &project.elements[0].geometry else {
        panic!("wrong geometry type");
    };
    let transform = Transform::new(block_model);
    let omf::BlockModel {
        orient: omf::Orient3 { .. },
        grid: omf::Grid3::Regular { .. },
        subblocks:
            Some(omf::Subblocks::Regular {
                subblocks,
                mode: Some(omf::SubblockMode::Octree),
                ..
            }),
    } = block_model
    else {
        panic!("wrong block model structure");
    };
    let omf::AttributeData::Category { values, .. } = &project.elements[0].attributes[0].data
    else {
        panic!("wrong attribute data type");
    };
    let subblocks = reader.array_regular_subblocks(subblocks).unwrap();
    let categories = reader.array_indices(values).unwrap();
    // Write only the points where the category index is 1.
    let mut out = File::create("./examples/bunny_blocks/bunny_centroids_out.csv").unwrap();
    writeln!(out, "x,y,z,dx,dy,dz").unwrap();
    for ((parent_index, subblock_range), category) in subblocks
        .map(Result::unwrap)
        .zip(categories.map(Result::unwrap))
    {
        let (centroid, size) = transform.get(parent_index, subblock_range);
        if category == Some(1) {
            writeln!(
                out,
                "{},{},{},{},{},{}",
                centroid.x, centroid.y, centroid.z, size.x, size.y, size.z
            )
            .unwrap();
        }
    }
}

struct Transform {
    origin: DVec3,
    mat: DMat3,
    subblock_size: DVec3,
    subblock_count: UVec3,
}

impl Transform {
    fn new(block_model: &omf::BlockModel) -> Self {
        let omf::BlockModel {
            orient: omf::Orient3 { origin, u, v, w },
            grid: omf::Grid3::Regular { size, .. },
            subblocks: Some(omf::Subblocks::Regular { count, .. }),
        } = block_model
        else {
            panic!("wrong block model structure");
        };
        let subblock_count = UVec3::from_array(*count);
        Self {
            origin: DVec3::from_array(*origin),
            mat: DMat3::from_cols_array_2d(&[*u, *v, *w]),
            subblock_size: DVec3::from_array(*size) / subblock_count.as_dvec3(),
            subblock_count,
        }
    }

    fn get(
        &self,
        [pi, pj, pk]: [u32; 3],
        [min_i, min_j, min_k, max_i, max_j, max_k]: [u32; 6],
    ) -> (DVec3, DVec3) {
        let parent = UVec3::new(pi, pj, pk);
        let min = UVec3::new(min_i, min_j, min_k);
        let max = UVec3::new(max_i, max_j, max_k);
        let a = (parent * self.subblock_count + min).as_dvec3() * self.subblock_size;
        let b = (parent * self.subblock_count + max).as_dvec3() * self.subblock_size;
        let size = b - a;
        let centroid = (a + b) / 2.0;
        (self.mat * centroid + self.origin, size)
    }
}
