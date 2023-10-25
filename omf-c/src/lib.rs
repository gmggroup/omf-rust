//! C wrapper for the [omf](omf) crate.
//!
//! Doesn't export any new Rust APIs. The C API is documented in the core OMF docs.

#![deny(unsafe_op_in_unsafe_fn)]

mod arrays;
mod attributes;
mod elements;
mod error;
mod ffi_tools;
mod from_omf;
mod image_data;
mod init_functions;
mod metadata;
mod omf1;
mod read_iterators;
mod reader;
mod to_omf;
mod validation;
mod writer;
mod writer_handle;

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::ptr::{null, null_mut};

    use crate::error::{omf_error, omf_error_free};
    use crate::init_functions::*;
    use crate::read_iterators::*;
    use crate::reader::*;
    use crate::writer::*;

    /// Same as the "pyramid" C example but written in Rust, to test the C API without
    /// any C code being involved.
    #[test]
    fn pyramid_rust() {
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
        const NAME: &str = "pyramid.omf\0";
        const PATH: &str = "../target/tmp/pyramid-rust.omf\0";
        const SURFACE_NAME: &str = "Pyramid surface\0";
        const LINE_SET_NAME: &str = "Pyramid edges\0";

        // Open.
        let writer = omf_writer_open(PATH.as_ptr().cast());
        // Init project.
        let mut project = omf_project_init(NAME.as_ptr().cast());
        project.name = "Test Project".as_ptr().cast();
        let proj_handle = omf_writer_project(writer, &project);
        // Add surface.
        let vertices = omf_writer_array_vertices32(writer, VERTICES.as_ptr(), VERTICES.len());
        let surface = omf_surface_init(
            vertices,
            omf_writer_array_triangles(writer, TRIANGLES.as_ptr(), TRIANGLES.len()),
        );
        let mut element = omf_element_init(SURFACE_NAME.as_ptr().cast());
        element.surface = &surface;
        element.color_set = true;
        element.color = [255, 128, 0, 255];
        let ele_handle = omf_writer_element(writer, proj_handle, &element);
        // Add metadata to that element.
        omf_writer_metadata_string(
            writer,
            ele_handle,
            "revision\0".as_ptr().cast(),
            "1.2\0".as_ptr().cast(),
        );
        let tags_handle = omf_writer_metadata_list(writer, ele_handle, "tags\0".as_ptr().cast());
        omf_writer_metadata_string(writer, tags_handle, null(), "foo\0".as_ptr().cast());
        omf_writer_metadata_string(writer, tags_handle, null(), "bar\0".as_ptr().cast());
        // Line-set element.
        let line_set = omf_line_set_init(
            vertices,
            omf_writer_array_segments(writer, SEGMENTS.as_ptr(), SEGMENTS.len()),
        );
        element = omf_element_init(LINE_SET_NAME.as_ptr().cast());
        element.line_set = &line_set;
        element.color_set = true;
        element.color = [0, 0, 0, 0];
        omf_writer_element(writer, proj_handle, &element);
        // Finish and close.
        omf_writer_finish(writer, null_mut());
        let error = omf_error();
        if !error.is_null() {
            let s = unsafe { CStr::from_ptr(error.read().message) };
            assert!(false, "Error: {s:?}");
            omf_error_free(error);
        }

        // Re-open to read.
        let reader = omf_reader_open(PATH.as_ptr().cast());
        let project = unsafe { omf_reader_project(reader, null_mut()).as_ref() }.unwrap();
        let name = unsafe { CStr::from_ptr(project.name) }.to_str().unwrap();
        assert_eq!(name, "Test Project");
        assert_eq!(project.n_elements, 2);
        let surface = unsafe { project.elements.as_ref() }.unwrap();
        let vertices_array = unsafe { surface.surface.as_ref().unwrap().vertices };
        let iter = omf_reader_array_vertices32_iter(reader, vertices_array);
        let mut vertices = Vec::new();
        let mut vertex = [0.0_f32; 3];
        while omf_vertices32_next(iter, vertex.as_mut_ptr()) {
            vertices.push(vertex);
        }
        assert_eq!(vertices, VERTICES);
    }
}
