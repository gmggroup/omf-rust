[![CI](https://github.com/gmggroup/omf-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/gmggroup/omf-rust/actions/workflows/ci.yml)
[![Audit](https://github.com/gmggroup/omf-rust/actions/workflows/audit.yml/badge.svg)](https://github.com/gmggroup/omf-rust/actions/workflows/audit.yml)

# OMF

A library for reading a writing files in Open Mining Format 2.0.
Also supports translating OMF 1 files to OMF 2.

OMF file version: 2.0-alpha.2

Crate version: 0.1.0-alpha.2

**Warning:** this is pre-release code.

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

## Compiling

First [install Rust](https://www.rust-lang.org/tools/install).
Run `cargo build --all --release` in the root directory to build the release version of the Rust
crate and C wrapper.
The C wrapper build will place `omf.h` and the platform-specific shared library files
(e.g.: `omfc.dll` and `omfc.dll.lib` for Windows) in `target/release`.

You can the `--release` argument off to build a debug version.
This may be useful for debugging C code that calls into OMF for example,
but it will be slow.

For the Rust tests, run `cargo test --all`.

To build and run the C examples:

1. Run `cargo build --all --release` first.
2. Change directory into `omf-c/examples/`.
3. Run `build.bat` on Windows or `build.sh` on Linux/MacOS.

This will build all examples, run them, and compare the results to the benchmarks.

## Documentation

The documentation is built with [MkDocs](https://www.mkdocs.org/).
To build locally:

1. Create and activate a Python virtual environment.
2. Change directory into `docs/`.
3. Run `build.bat` on Windows or `build.sh` on Linux/MacOS.

This will install the required dependencies, then build the file format, Rust, and C documentation into `site/`.
