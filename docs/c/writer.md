# Writer

## OmfHandle

An opaque pointer used for creating nested structures in an OMF file.
Many [OmfWriter](#OmfWriter) functions return a handle that points to the newly created object,
while others take that handle and create their new object inside the existing object.

Not all handle types are valid in all places.
You'll get an error at run-time if you pass the wrong type.
Handles belong to a specific writer and remain valid until [omf_writer_finish](#omf_writer_finish) is called,
you don't need to free them yourself and you shouldn't try to use them on a second writer.


## OmfWriter

The class used for writing OMF files.

Typical usage pattern is:

1. Create the writer object using [omf_writer_open](#omf_writer_open).
1. Fill in an [OmfProject](project.md#OmfProject) struct then call [omf_writer_project](#omf_writer_project)
to add it to the writer.
1. For each element you want to store:
    1. Write the arrays and images.
    1. Fill in the required struct with the array pointers and other details then add it to the project.
    1. Repeat for the attributes, adding them to the newly created element.
1. Call [omf_writer_finish](#omf_writer_finish) to validate the data,
finish writing the file, and free the writer.

You can also fill in the `OmfProject::elements` and `OmfElement::attributes` arrays all at once,
but that will often lead to messy code.

### Methods

#### omf_writer_open

```c
OmfWriter *omf_writer_open(const char *path);
```

Creates a new writer.
The `path` string must be UTF-8 encoded.
The file will be created if it does not exist, or truncated and overwritten if it does.

Returns the new writer, or null on error.
Pass the writer to `omf_writer_finish` or `omf_writer_cancel` when you're finished with it.

#### omf_writer_finish

```c
bool omf_writer_finish(OmfWriter *writer, OmfValidation **validation);
```

Validates the contents, writes the file index, closes the file, and frees the writer.

Validation messages are stored in `validation` if it is non-null,
or written to `stdout` is it is null. Call `omf_validation_free` to free those messages.

Returns false on error; the writer is still freed.

#### omf_writer_compression

```c
int32_t omf_writer_compression(OmfWriter *writer);
```

Returns the compression level that the writer will use, or -1 on error.

#### omf_writer_set_compression

```c
bool omf_writer_set_compression(OmfWriter *writer, int32_t compression);
```

Sets the compression level that the writer will use.
Pass an integer between 1 for fastest and 9 for most compressed, or -1 to use the default.

Returns false on error.

#### omf_writer_cancel

```c
bool omf_writer_cancel(OmfWriter *writer);
```

Frees the writer without finishing it.
The partially written file will be deleted.

Returns false if file deletion fails; the writer is still freed.

#### omf_writer_project

```c
OmfHandle *omf_writer_project(OmfWriter *writer, const OmfProject *project);
```

Copies the contents of `project` to the writer.
All `OmfArray` pointers inside `project` must have been previously written to this writer.

Returns the project handle, or null on error.

#### omf_writer_element

```c
OmfHandle *omf_writer_element(OmfWriter *writer,
                              OmfHandle *handle,
                              const OmfElement *element);
```

Copies the contents of `element` into list elements in the object identified by `handle`,
which can be either the project or a composite element.

Returns a handle to the new element, or null on error.

#### omf_writer_attribute

```c
OmfHandle *omf_writer_attribute(OmfWriter *writer,
                                OmfHandle *handle,
                                const OmfAttribute *attribute);
```

Copies the contents of `attribute` into the list the attributes of the object identified by `handle`,
which can be either an element or a category attribute.

Returns a handle to the new attribute, or null on error.

#### omf_writer_metadata_null

```c
bool omf_writer_metadata_null(OmfWriter *writer, OmfHandle *handle, const char *name);
```

Adds a metadata item with null value.
The `handle` can be the project, an element, an attribute, or a metadata list or object.
`name` must be a UTF-8 encoded string, or null if adding to a metadata list.

Returns false on error.

#### omf_writer_metadata_boolean

```c
bool omf_writer_metadata_boolean(OmfWriter *writer,
                                 OmfHandle *handle,
                                 const char *name,
                                 bool value);
```                                 

Adds a metadata item with boolean value.
The `handle` can be the project, an element, an attribute, or a metadata list or object.
`name` must be a UTF-8 encoded string, or null if adding to a metadata list.

Returns false on error.

#### omf_writer_metadata_number

```c
bool omf_writer_metadata_number(OmfWriter *writer,
                                OmfHandle *handle,
                                const char *name,
                                double value);
```                                

Adds a metadata item with double value.
The `handle` can be the project, an element, an attribute, or a metadata list or object.
`name` must be a UTF-8 encoded string, or null if adding to a metadata list.

Returns false on error.

#### omf_writer_metadata_string

```c
bool omf_writer_metadata_string(OmfWriter *writer,
                                OmfHandle *handle,
                                const char *name,
                                const char *value);
```                                

Adds a metadata item with string value.
The `handle` can be the project, an element, an attribute, or a metadata list or object.
`name` must be a UTF-8 encoded string, or null if adding to a metadata list.

Returns false on error.

#### omf_writer_metadata_list

```c
OmfHandle *omf_writer_metadata_list(OmfWriter *writer,
                                    OmfHandle *handle,
                                    const char *name);
```                                     

Adds a metadata item with empty list value.
The `handle` can be the project, an element, an attribute, or a metadata list or object.
`name` must be a UTF-8 encoded string, or null if adding to a metadata list.

Returns a metadata object handle, or null on error.
When adding metadata inside this handle names will be ignored and order maintained.

#### omf_writer_metadata_object

```c
OmfHandle *omf_writer_metadata_object(OmfWriter *writer,
                                      OmfHandle *handle,
                                      const char *name);
```

Adds a metadata item with empty object value.
The `handle` can be the project, an element, an attribute, or a metadata list or object.
`name` must be a UTF-8 encoded string, or null if adding to a metadata list.

Returns a metadata object handle, or null on error.
When adding further values inside this handle the value names will be used to create
a collection of key/value pairs.

#### omf_writer_image_png8

```c
const OmfArray *omf_writer_image_png8(OmfWriter *writer,
                                      uint32_t width,
                                      uint32_t height,
                                      OmfImageMode mode,
                                      const uint8_t *pixels);
```

Writes an image in PNG encoding. This method supports 8 bits per channel,
use [omf_writer_image_png16](#omf_writer_image_png16) for 16-bit images.
The length of `pixels` must be `width * height * mode`.

Pixel order is across the top row, then the second, and so on.
Channel order is ($r_0, g_0, b_0, a_0, r_1, g_1, b_1, a_1, ...$)
with the exact channels present depending `mode`.
Don't use any padding or row alignment.

Returns the new `OmfArray*` pointer, or null on error.

#### omf_writer_image_png16

```c
const OmfArray *omf_writer_image_png16(OmfWriter *writer,
                                       uint32_t width,
                                       uint32_t height,
                                       OmfImageMode mode,
                                       const uint16_t *pixels);
```

16-bit version of [omf_writer_image_png8](#omf_writer_image_png8). 

#### omf_writer_image_jpeg

```c
const OmfArray *omf_writer_image_jpeg(OmfWriter *writer,
                                      uint32_t width,
                                      uint32_t height,
                                      const uint8_t *pixels,
                                      uint32_t quality);
```

Writes an image to the OMF file in JPEG encoding. JPEG only supports 8-bit RGB.
The length of `pixels` must be `width * height * 3`.

Compared to PNG, JPEG encoding will give much smaller file sizes at the cost of reduced quality.
It is best used for photos or high resolution scans where the fine detail is mostly noise
and PNG encoding would be excessively large.
If your image comes from an existing JPEG file then you should use
[omf_writer_image_bytes](#omf_writer_image_bytes) or [omf_writer_image_file](#omf_writer_image_file)
instead; repeatedly encoding will further reduce the quality.

Returns the new `OmfArray*` pointer, or null on error.

#### omf_writer_image_bytes

```c
const OmfArray *omf_writer_image_bytes(OmfWriter *writer,
                                       const uint8_t *bytes,
                                       size_t n_bytes);
```

Writes an image from bytes already in PNG or JPEG encoding.

Returns the new `OmfArray*` pointer, or null on error.

#### omf_writer_image_file

```c
const OmfArray *omf_writer_image_file(OmfWriter *writer, const char *path);
```

Writes an existing PNG or JPEG image file. The `path` string must be UTF-8 encoded.

Returns the new `OmfArray*` pointer, or null on error.

#### omf_writer_array_bytes

```c
const OmfArray *omf_writer_array_bytes(OmfWriter *writer,
                                              OmfArrayType array_type,
                                              uint64_t item_count,
                                              const char *bytes,
                                              size_t n_bytes);
```

Writes a previously compressed Parquet array.
Useful if you want to use an alternative Parquet implementation,
or if you're copying arrays from one file to another.
Check for the OMF file format documentation for the Parquet schema that each array type much match.

#### omf_writer_array_…

```c
const OmfArray *omf_writer_array_scalars64            (OmfWriter *writer,
                                                       const double *values,
                                                       size_t length);
const OmfArray *omf_writer_array_scalars32            (OmfWriter *writer,
                                                       const float *values,
                                                       size_t length);
const OmfArray *omf_writer_array_vertices64           (OmfWriter *writer,
                                                       const double (*values)[3],
                                                       size_t length);
const OmfArray *omf_writer_array_vertices32           (OmfWriter *writer,
                                                       const float (*values)[3],
                                                       size_t length);
const OmfArray *omf_writer_array_segments             (OmfWriter *writer,
                                                       const uint32_t (*values)[2],
                                                       size_t length);
const OmfArray *omf_writer_array_triangles            (OmfWriter *writer,
                                                       const uint32_t (*values)[3],
                                                       size_t length);
const OmfArray *omf_writer_array_names                (OmfWriter *writer,
                                                       const char *const *values,
                                                       size_t length);
const OmfArray *omf_writer_array_gradient             (OmfWriter *writer,
                                                       const uint8_t (*values)[4],
                                                       size_t length);
const OmfArray *omf_writer_array_texcoords64          (OmfWriter *writer,
                                                       const double (*values)[2],
                                                       size_t length);
const OmfArray *omf_writer_array_texcoords32          (OmfWriter *writer,
                                                       const float (*values)[2],
                                                       size_t length);
const OmfArray *omf_writer_array_boundaries_float32   (OmfWriter *writer,
                                                       const float *values,
                                                       const bool *inclusive,
                                                       size_t length);
const OmfArray *omf_writer_array_boundaries_float64   (OmfWriter *writer,
                                                       const double *values,
                                                       const bool *inclusive,
                                                       size_t length);
const OmfArray *omf_writer_array_boundaries_int64     (OmfWriter *writer,
                                                       const int64_t *values,
                                                       const bool *inclusive,
                                                       size_t length);
const OmfArray *omf_writer_array_boundaries_date      (OmfWriter *writer,
                                                       const int64_t *values,
                                                       const bool *inclusive,
                                                       size_t length);
const OmfArray *omf_writer_array_boundaries_date_time (OmfWriter *writer,
                                                       const int64_t *values,
                                                       const bool *inclusive,
                                                       size_t length);
const OmfArray *omf_writer_array_regular_subblocks    (OmfWriter *writer,
                                                       const uint32_t (*parents)[3],
                                                       const uint32_t (*corners)[6],
                                                       size_t length);
const OmfArray *omf_writer_array_freeform_subblocks32 (OmfWriter *writer,
                                                       const uint32_t (*parents)[3],
                                                       const float (*corners)[6],
                                                       size_t length);
const OmfArray *omf_writer_array_freeform_subblocks64 (OmfWriter *writer,
                                                       const uint32_t (*parents)[3],
                                                       const double (*corners)[6],
                                                       size_t length);
const OmfArray *omf_writer_array_numbers_float32      (OmfWriter *writer,
                                                       const float *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_numbers_float64      (OmfWriter *writer,
                                                       const double *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_numbers_int64        (OmfWriter *writer,
                                                       const int64_t *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_numbers_date         (OmfWriter *writer,
                                                       const int32_t *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_numbers_date_time    (OmfWriter *writer,
                                                       const int64_t *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_indices              (OmfWriter *writer,
                                                       const uint32_t *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_vectors32x2          (OmfWriter *writer,
                                                       const float (*values)[2],
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_vectors64x2          (OmfWriter *writer,
                                                       const double (*values)[2],
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_vectors32x3          (OmfWriter *writer,
                                                       const float (*values)[3],
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_vectors64x3          (OmfWriter *writer,
                                                       const double (*values)[3],
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_text                 (OmfWriter *writer,
                                                       const char *const *values,
                                                       size_t length);
const OmfArray *omf_writer_array_booleans             (OmfWriter *writer,
                                                       const bool *values,
                                                       const bool *mask,
                                                       size_t length);
const OmfArray *omf_writer_array_colors               (OmfWriter *writer,
                                                       const uint8_t (*values)[4],
                                                       const bool *mask,
                                                       size_t length);
```

Functions for writing OMF arrays, pulling data from contiguous C arrays.

For nullable types the `mask` array may be null if all values are valid.
Otherwise it should be the same length as the values array,
containing true where the value is null or false where it is valid.

Strings here are always nul-terminated.
Only a limited set of input types is supported.
The iterator API below can provide more flexibility, and avoid copying in some cases.
