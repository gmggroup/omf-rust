# OMF

A library for reading a writing files in Open Mining Format 2.0.

## What is OMF

OMF is an open-source serialization format and library to support data interchange
across the entire mining community.
Its goal is to standardize file formats and promote collaboration.

This repository provides a file format specification and a Rust library for reading and writing files,
plus a wrapper to use that library from C.

## What OMF Stores

### Elements

- Points.
- Line segments.
- Triangulated surfaces.
- Grid surfaces.
    - Regular or tensor grid spacing.
    - Any orientation.
- Block models, with optional sub-blocks.
    - Regular or tensor grid spacing.
    - Any orientation.
    - Regular sub-blocks that lie on a grid within their parent, with octree or arbitrary layout.
    - Free-form sub-blocks that don't lie on any grid.
- Composite elements made out of any of the above.

### Attributes

- Floating-point or signed integer values.
- Date and date-time values.
- Category values, storing an index used to look up name, color, or other sub-attributes.
- Boolean or filter values.
- 2D and 3D vectors.
- Text values.
- Color values.
- Projected texture images.
- UV mapped texture images.

Attributes values can be valid or null.
They can be attached to different parts of each element type,
such as the vertices vs. faces of a surface,
or the parent blocks vs. sub-blocks of a block model.
