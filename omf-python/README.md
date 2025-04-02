# Getting Started

> **Warning:**
> These Python bindings are incomplete.
> The API may experience breaking changes before it is finished.

## Missing Parts

- Composite geometry is not supported.

- Writing files is not supported.

- Can only read real files, specified by name. The Rust code is generic now, so it will be
  possible to support Python `io.BytesIO` objects as well. Will need to make sure that
  performance is still acceptable though.

- PyO3 now supports wrapping rich enums.
  Use that support rather than wrapping each variant as a separate struct.

## Building

To build the Python bindings, run these commands in the project root:

```sh
cd omf-python
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
maturin develop
```

Only Python 3 is supported.
Depending on your operating system, you may need to use `python3` rather than `python`
to select the correct version.

You can then interact with the Python API locally like this:

```sh
# Generate the one of everything omf:
$ cargo test --all
$ python

Python 3.12.5 (main, Aug  6 2024, 19:08:49) [GCC 11.4.0] on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import omf2
>>> reader = omf2.Reader("../target/tmp/one_of_everything.omf")
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

This will create a wheel in `./target/wheels`.

Comments and types in the python bindings code don't automatically get converted into python doc strings/typing information.
To generate the python .pyi stub file:
```sh
cd omf-python
cargo run --bin stub_gen
```

This will create a file `omf2.pyi` which will get included automatically the next time you run `maturin develop`.
Afterwards you should be able to see comments and typing information about omf2 in your editor.

Some test OMF files need to be generated so that all features can be tested via `pytest`. If you need to make more test
files, or modify existing ones:

```sh
cd omf-python
cargo run --bin generate_test_data
```
