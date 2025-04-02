# Home

Version 0.2.0-beta.1

Specification and library for Open Mining Format version 2,
a standard for mining data interchange backed by the
[Global Mining Guidelines Group](https://gmggroup.org).

> WARNING:
> This is an alpha release of OMF 2. The storage format and libraries might be changed in
> backward-incompatible ways and are not subject to any SLA or deprecation policy.
>
> Further, this code is unfinished and may not be secure.
> Don't use it to open files you don't trust, and don't use it in production yet.


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


## Using OMF

See the [getting started](start.md) page.
