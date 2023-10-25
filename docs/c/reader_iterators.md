# Reader Iterators

## Reading Iterator Objects

Each array type has a matching iterator type.
These can be used to iterate over the values of an array without allocating temporary buffers.

Note that these objects are **not thread safe**.
You must create it, iterator over it, and free it all on the same thread.
Iterators may outlast the reader object; they have their own copy of the open file handle.

### Methods

#### omf_…_next

```c
bool omf_scalars32_next(OmfScalars32 *iter, float *scalar);
bool omf_scalars64_next(OmfScalars64 *iter, double *scalar);
bool omf_vertices32_next(OmfVertices32 *iter, float vertex[3]);
bool omf_vertices64_next(OmfVertices64 *iter, double vertex[3]);
bool omf_segments_next(OmfSegments *iter, uint32_t segment[2]);
bool omf_triangles_next(OmfTriangles *iter, uint32_t triangles[3]);
bool omf_gradient_next(OmfGradient *iter, uint8_t rgba[4]);
bool omf_texcoords32_next(OmfTexcoords32 *iter, float uv[2]);
bool omf_texcoords64_next(OmfTexcoords64 *iter, double uv[2]);
bool omf_boundaries_float32_next(OmfBoundariesFloat32 *iter, float *value, bool *inclusive);
bool omf_boundaries_float64_next(OmfBoundariesFloat64 *iter, double *value, bool *inclusive);
bool omf_boundaries_int64_next(OmfBoundariesInt64 *iter, int64_t *value, bool *inclusive);
bool omf_numbers_float32_next(OmfNumbersFloat32 *iter, float *number, bool *is_null);
bool omf_numbers_float64_next(OmfNumbersFloat64 *iter, double *number, bool *is_null);
bool omf_numbers_int64_next(OmfNumbersInt64 *iter, int64_t *number, bool *is_null);
bool omf_indices_next(OmfIndices *iter, uint32_t *index, bool *is_null);
bool omf_booleans_next(OmfBooleans *iter, bool *boolean, bool *is_null);
bool omf_omf_colors_next(OmfColors *iter, uint8_t rgba[4], bool *is_null);
bool omf_names_next(OmfNames *iter, const char **string, size_t *len);
bool omf_text_next(OmfText *iter, const char **string, size_t *len);
```

These functions must be called from the same thread that created the iterator.

Retrieve the next value from an iterator.
If another item is available, fills in the output arguments are returns true.
Returns false without changing the output arguments if no more items are available.

Strings will always be nul-terminated but the length is also output for convenience.
The string buffer will be valid until `next` is called again.

These iterators will end early if something fails.
Use `omf_error()` to distinguish normal termination for an error.

#### omf_…_free

```c
void omf_scalars32_free(OmfScalars32 *iter);
void omf_scalars64_free(OmfScalars64 *iter);
void omf_vertices32_free(OmfVertices32 *iter);
void omf_vertices64_free(OmfVertices64 *iter);
void omf_segments_free(OmfSegments *iter);
void omf_triangles_free(OmfTriangles *iter);
void omf_gradient_free(OmfGradient *iter);
void omf_texcoords32_free(OmfTexcoords32 *iter);
void omf_texcoords64_free(OmfTexcoords64 *iter);
void omf_boundaries_float64_free(OmfBoundariesFloat64 *iter);
void omf_boundaries_float32_free(OmfBoundariesFloat32 *iter);
void omf_boundaries_int64_free(OmfBoundariesInt64 *iter);
void omf_numbers_float32_free(OmfNumbersFloat32 *iter);
void omf_numbers_float64_free(OmfNumbersFloat64 *iter);
void omf_numbers_int64_free(OmfNumbersInt64 *iter);
void omf_indices_free(OmfIndices *iter);
void omf_booleans_free(OmfBooleans *iter);
void omf_colors_free(OmfColors *iter);
void omf_names_free(OmfNames *iter);
void omf_text_free(OmfText *iter);
```

Frees an iterator, releasing all resources it is holding.
Must be called from the same thread that created the iterator.

## OmfReader

### Methods

#### omf_reader_array_…_iter

```c
OmfScalars32 *omf_reader_array_scalars32_iter(OmfReader *reader, const OmfArray *array);
OmfScalars64 *omf_reader_array_scalars64_iter(OmfReader *reader, const OmfArray *array);
OmfVertices32 *omf_reader_array_vertices32_iter(OmfReader *reader, const OmfArray *array);
OmfVertices64 *omf_reader_array_vertices64_iter(OmfReader *reader, const OmfArray *array);
OmfSegments *omf_reader_array_segments_iter(OmfReader *reader, const OmfArray *array);
OmfTriangles *omf_reader_array_triangles_iter(OmfReader *reader, const OmfArray *array);
OmfNames *omf_reader_array_names_iter(OmfReader *reader, const OmfArray *array);
OmfGradient *omf_reader_array_gradient_iter(OmfReader *reader, const OmfArray *array);
OmfTexcoords32 *omf_reader_array_texcoords32_iter(OmfReader *reader, const OmfArray *array);
OmfTexcoords64 *omf_reader_array_texcoords64_iter(OmfReader *reader, const OmfArray *array);
OmfBoundariesFloat32 *omf_reader_array_boundaries_float32_iter(OmfReader *reader, const OmfArray *array);
OmfBoundariesFloat64 *omf_reader_array_boundaries_float64_iter(OmfReader *reader, const OmfArray *array);
OmfBoundariesInt64 *omf_reader_array_boundaries_int64_iter(OmfReader *reader, const OmfArray *array);
OmfRegularSubblocks *omf_reader_array_regular_subblocks_iter(OmfReader *reader, const OmfArray *array);
OmfFreeformSubblocks32 *omf_reader_array_freeform_subblocks32_iter(OmfReader *reader, const OmfArray *array);
OmfFreeformSubblocks64 *omf_reader_array_freeform_subblocks64_iter(OmfReader *reader, const OmfArray *array);
OmfNumbersFloat32 *omf_reader_array_numbers_float32_iter(OmfReader *reader, const OmfArray *array);
OmfNumbersFloat64 *omf_reader_array_numbers_float64_iter(OmfReader *reader, const OmfArray *array);
OmfNumbersInt64 *omf_reader_array_numbers_int64_iter(OmfReader *reader, const OmfArray *array);
OmfIndices *omf_reader_array_indices_iter(OmfReader *reader, const OmfArray *array);
OmfVectors32x2 *omf_reader_array_vectors32x2_iter(OmfReader *reader, const OmfArray *array);
OmfVectors64x2 *omf_reader_array_vectors64x2_iter(OmfReader *reader, const OmfArray *array);
OmfVectors32x3 *omf_reader_array_vectors32x3_iter(OmfReader *reader, const OmfArray *array);
OmfVectors64x3 *omf_reader_array_vectors64x3_iter(OmfReader *reader, const OmfArray *array);
OmfText *omf_reader_array_text_iter(OmfReader *reader, const OmfArray *array);
OmfBooleans *omf_reader_array_booleans_iter(OmfReader *reader, const OmfArray *array);
OmfColors *omf_reader_array_colors_iter(OmfReader *reader, const OmfArray *array);
```

Get an iterator for reading an array from the OMF file.
Returns null on error.
Pass the result to the matching free function when you're finished with it.

All floating-point arrays will automatically cast from 32-bit to 64-bit,
but not the reverse because that would lose precision.

For number arrays,
dates can be read into `int64_t` or `double` as the number of whole days since the epoch.
Date-time can be read into `int64_t` as the number of microseconds since the epoch,
or into `double` as the number of seconds since the epoch, including a fractional component,
with a small loss of precision.
