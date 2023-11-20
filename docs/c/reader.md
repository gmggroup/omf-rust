# Reader

## OmfFileVersion

```c
typedef struct {
    uint32_t major;
    uint32_t minor;
} OmfFileVersion;
```

The major and minor version numbers returned by [omf_reader_version](#omf_reader_version).


## OmfLimits

```c
typedef struct {
    uint64_t json_bytes;
    uint64_t image_bytes;
    uint32_t image_dim;
    uint32_t validation;
} OmfLimits;
```

Contains the safety limits used by `OmfReader` when reading files.
For all fields zero means unlimited.

Limits are set on a per-reader basis using [omf_reader_set_limits](#omf_reader_set_limits).

> WARNING:
> Running without any limits is not recommended.
> A file could be maliciously crafted to consume excessive system resources when read and decompressed,
> leading to a denial of service attack.

### Fields

json_bytes: `uint64_t`
: Maximum uncompressed size for the JSON index. Default is 1 MB.

image_bytes: `uint64_t`
: Maximum memory to use when decoding an image.
Default is 1 GB on 32-bit systems or 16 GB on 64-bit systems.

image_dim: `uint32_t`
: Maximum image width or height in pixels, default unlimited.

validation: `uint32_t`
: Maximum number of validation messages.
Errors beyond this limit will be discarded.
Default is 100.


## OmfReader

The class used for reading OMF files.

Typical usage pattern is:

1. Create the reader object with [omf_reader_open](#omf_reader_open).
1. Optional: retrieve the file version with [omf_reader_version](#omf_reader_version).
1. Optional: adjust the limits with [omf_reader_set_limits](#omf_reader_set_limits).
1. Read the project from the file with [omf_reader_project](#omf_reader_project).
1. Iterate through the project's contents to find the elements and attributes you want to load.
1. For each of those items load the array or image data.

### Methods

#### omf_reader_open

```c
OmfReader *omf_reader_open(const char *path);
```

Attempts to opens the given path as an OMF file.
The `path` string must be UTF-8 encoded.

Returns a new reader object on success or null on error.
Pass the returned pointer to `omf_reader_close` when you're finished everything inside it.

#### omf_reader_close

```c
bool omf_reader_close(OmfReader *reader);
```

Closes and frees a reader returned by `omf_reader_open`.
Does nothing if `reader` is null.
Returns false on error.

You mustn't use `reader` or anything belonging to it after this call.

#### omf_reader_version

```c
OmfFileVersion omf_reader_version(OmfReader *reader);
```

Returns the OMF version of the file opened with this reader.

#### omf_reader_limits

```c
struct OmfLimits omf_reader_limits(OmfReader *reader);
```

Returns the current safety limits, or the default limits if `reader` is null.

#### omf_reader_set_limits

```c
bool omf_reader_set_limits(OmfReader *reader, const OmfLimits *limits);
```

Sets the safety limits for this reader.
If `limits` is null then the default limits are restored.

#### omf_reader_project

```c
const OmfProject *omf_reader_project(OmfReader *reader, OmfValidation **validation);
```

Reads, validates, and returns the project from this OMF file.
Returns null on error.
The returned pointer belongs to the reader and does not need to be freed separately.
You may only call this function once on a given reader.

If validation fails, or succeeds but produces warnings,
`validation` will be modified to point to a new `OmfValidation` struct containing the validation messages.
If `validation` is null then these messages will be printed to `stdout` instead.

#### omf_reader_image

```c
OmfImageData *omf_reader_image(OmfReader *reader, const OmfArray *image);
```

Reads and returns an image from the OMF file. Returns null on error. 

Pass the returned pointer to `omf_image_free` when you're finished with it.

#### omf_reader_array_info

```c
OmfArrayInfo omf_reader_array_info(OmfReader *reader, const OmfArray *array);
```

Returns information about an array. See [`OmfArrayInfo`](./arrays.md#OmfArrayInfo).
The `array_type` will be `-1` if an error occurs.

#### omf_reader_array_bytes

```c
bool omf_reader_array_bytes(struct OmfReader *reader,
                            const struct OmfArray *array,
                            char *output,
                            size_t n_output);
```

Reads an array without decompressing it.
Call `omf_reader_array_info` to get the compressed size so you can allocate a large enough buffer.
Useful if you want to use an alternative Parquet implementation,
or if you're copying arrays from one file to another.

#### omf_reader_array_…

```c
bool omf_reader_array_scalars64            (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double *values,
                                            size_t n_values);
bool omf_reader_array_scalars32            (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            float *values,
                                            size_t n_values);
bool omf_reader_array_vertices64           (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double (*values)[3],
                                            size_t n_values);
bool omf_reader_array_vertices32           (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            float (*values)[3],
                                            size_t n_values);
bool omf_reader_array_segments             (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint32_t (*values)[2],
                                            size_t n_values);
bool omf_reader_array_triangles            (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint32_t (*values)[3],
                                            size_t n_values);
bool omf_reader_array_gradient             (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint8_t (*values)[4],
                                            size_t n_values);
bool omf_reader_array_texcoords64          (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double (*values)[2],
                                            size_t n_values);
bool omf_reader_array_texcoords32          (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            float (*values)[2],
                                            size_t n_values);
bool omf_reader_array_boundaries_float64   (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double *values,
                                            bool *inclusive,
                                            size_t n_values);
bool omf_reader_array_boundaries_int64     (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            int64_t *values,
                                            bool *inclusive,
                                            size_t n_values);
bool omf_reader_array_regular_subblocks    (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint32_t (*parents)[3],
                                            uint32_t (*corners)[6],
                                            size_t n_values);
bool omf_reader_array_freeform_subblocks64 (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint32_t (*parents)[3],
                                            double (*corners)[6],
                                            size_t n_values);
bool omf_reader_array_freeform_subblocks32 (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint32_t (*parents)[3],
                                            float (*corners)[6],
                                            size_t n_values);
bool omf_reader_array_numbers_float64      (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double *values,
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_numbers_float32      (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            float *values,
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_indices              (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint32_t *values,
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_vectors32x2          (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            float (*values)[2],
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_vectors64x2          (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double (*values)[2],
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_vectors32x3          (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            float (*values)[3],
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_vectors64x3          (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            double (*values)[3],
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_booleans             (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            bool *values,
                                            bool *mask,
                                            size_t n_values);
bool omf_reader_array_colors               (struct OmfReader *reader,
                                            const struct OmfArray *array,
                                            uint8_t (*values)[4],
                                            bool *mask,
                                            size_t n_values);
```
