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

This is a work in progress. To contribute to building Python bindings, in the project root:

```sh
cd omf-python
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
maturin develop
```

You can then interact with the Python API locally like this:

```sh
# Generate the one of everything omf:
$ cargo test --all
$ python

Python 3.12.5 (main, Aug  6 2024, 19:08:49) [GCC 11.4.0] on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import omf_python
>>> reader = omf_python.Reader("../target/tmp/one_of_everything.omf")
>>> for element in reader.project.elements:
...     print(element.geometry.get_type())
...
Surface
PointSet
LineSet
GridSurface
BlockModel
BlockModel
BlockModel(sub-blocked)
BlockModel(sub-blocked)
Composite
Surface
>>> reader.project.elements[1].geometry.get_object().get_origin()
[0.0, 0.0, 0.0]
```

You can build a release version using:

```sh
maturin build --release
```

This will create a wheel in `./target/wheels`

Comments and types in the python bindings code don't automatically get converted into python doc strings/typing information.
To generate the python .pyi stub file:
```sh
cd omf-python
cargo run --bin stub_gen
```

This will create a file `omf_python.pyi` which will get included automatically the next time you run `maturin develop`.
Afterwards you should be able to see comments and typing information about omf_python in your editor.

One you've generated `omf_python.pyi` you can build the html API docs:

```sh
cd omf-python/docs
make html
```

You can then view the python API documentation here: [here](../omf-python/docs/build/html/index.html).

Some test OMF files need to be generated so that all features can be tested via `pytest`. If you need to make more test
files, or modify existing ones:

```sh
cd omf-python
cargo run --bin generate_test_data
```

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
