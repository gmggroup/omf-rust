# Changelog

<!-- add future changes here, most recent version first -->

## Version 2.0

- New Rust library and C API.
The new code focuses on security, ease of use, and performance.

- Use ZIP as the container format, with an identifying comment.

- Use Apache Parquet to compress array data.

- The JSON data is now compressed.

- The JSON structure has been reworked into something more predictable,
consistent, and flexible. A JSON [schema](schema_index.md) is provided as documentation
and specification.

- Arbitrary JSON metadata is now supported on the project, elements, and attributes.

- Added JPEG image support.

- Added boolean-valued attributes.

- Added UV mapped textures.

- All attributes now have a standard representation for null values.
Going forward you should avoid using NaN or other flag values like -9999 for nulls.

- Block models can now have sub-blocks.
Regular sub-blocks lie on a grid within the parent block,
while free-form sub-blocks can be anywhere.

- Grid surfaces can now be regularly spaced.
