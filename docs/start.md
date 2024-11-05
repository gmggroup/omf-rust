# Getting Started

## Overview of an OMF File

An OMF file is a ZIP archive with an identifying comment.
It contains a JSON [index](format.md#json-index) document,
plus other files for [arrays](format.md#arrays) and [images](format.md#images) that the index refers to.

The root object of the JSON document is a **project**.
The project contains one or more **elements** each of which describes a separate object,
like a set of points, a triangulated surface, or a block model.
Each element can have any number of **attributes** which describe things like assay measurements on points,
colors on triangles, and estimation outputs on blocks.

Bulk data, like **images** or **arrays** of vertex locations,
are not stored as JSON but as separate files within the archive.
The JSON data refers to each data file by name,
and contains details for linking them together into rich objects.
Images may use PNG or JPEG encoding, while arrays use Apache Parquet encoding.

> WARNING:
> When reading OMF files, beware of "zip bombs" where data is maliciously crafted to expand to an
> excessive size when decompressed, leading to a potential denial of service attack.
> Use the limits provided by the C and Rust APIs, and check sizes before allocating memory.


## Rust API

See the [Rust API documentation](rust/omf/index.html).


## C API

See the [C API documentation](c/index.md).


## Python API

See the [Python API documentation](../omf-python/docs/build/html/index.html).

## Write Your Own

To create your own reading or writing code,
start with the [file format](format.md) specification,
[JSON schema](schema_index.md),
and [Parquet schema](parquet.md) documentation.

How you proceed depends on the language you're working in.
You will probably want to start by finding good libraries for:

- UTF-8 character encoding.
- JSON parsing.
- Deflate compression and decompression, a.k.a. Zlib.
- Apache Parquet compression and decompression.
- Reading and writing PNG and JPEG images.

> WARNING:
> Make sure that these libraries are secure against malicious data,
> and keep track of any security updates for them.
