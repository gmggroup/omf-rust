# OMF File Format

The basic structure of an OMF 2 file is a [ZIP archive](https://en.wikipedia.org/wiki/ZIP_(file_format)),
with ZIP64 extensions.

The files in the archive are stored without compression.
Each type of file uses a separate and data-specific compression separate from the ZIP archive,
and compressing them again would be a waste of time.

The top-level zip comment will contain the format name and version,
such as `Open Mining Format 2.0`.
Due to the structure of ZIP files that comment will always be the final bytes in the file.
A ZIP created by another application won't have this comment so won't be recognized as an OMF file.

The OMF file will contain three types of files: the JSON index, arrays, and images.


## JSON Index

The index is a gzip-compressed JSON document called `index.json.gz`.
This file is required, and describes the project, elements, and attributes in the OMF file.

See the [JSON schema](schema_index.md) documentation for a specification of the index.


## Arrays

Arrays are stored in (Apache Parquet)[https://en.wikipedia.org/wiki/Apache_Parquet] format,
using a `.parquet` extension.
Each array of vertex locations, triangles, etc. will be in a separate file within the archive.
The JSON index will refer to array files by name.

Several different array types are used,
see the [Parquet schema](parquet.md) documentation for a specification of each one.


## Images

Images are encoded in either [PNG](https://en.wikipedia.org/wiki/PNG)
or [JPEG](https://en.wikipedia.org/wiki/JPEG) encoded files,
which should have `.png` and `.jpg` extensions respectively.
The JSON index will refer to image files by name.

PNG encoding can store grayscale, grayscale-alpha, RGB or RGBA data with 8 or 16 bits per channel.
The compression is lossless.

JPEG encoding can store only 8-bit per channel RGB.
They use lossy compression which won't preserve fine details but gives a smaller file size.
This is suitable for high-resolution maps and scans where it will give much smaller file sizes.


## Versions

The current version is 2.0.

When new features are added the **minor** version number will be incremented,
to 2.1, then 2.2, and so on.

If features are ever removed,
or changed such that the format can't store something it used to store,
the **major** version number will be incremented.
This won't be done lightly, so it is unlikely to ever happen.

When writing files the OMF library will use the oldest version possible,
down to 2.0, based on what is being stored.
Even if the library can write version 2.2 files, if you don't use any of the new features a
2.1 or 2.0 file may be written.
