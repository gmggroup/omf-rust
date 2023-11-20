# Writer Iterators

## Omf…Source

```c
typedef bool (*OmfScalar32Source)(void *object, float *scalar);
typedef bool (*OmfScalar64Source)(void *object, double *scalar);
typedef bool (*OmfVertex32Source)(void *object, float vertex[3]);
typedef bool (*OmfVertex64Source)(void *object, double vertex[3]);
typedef bool (*OmfSegmentSource)(void *object, uint32_t segment[2]);
typedef bool (*OmfTriangleSource)(void *object, uint32_t triangle[3]);
typedef bool (*OmfNameSource)(void *object, const char **value);
typedef bool (*OmfGradientSource)(void *object, uint8_t rgba[4]);
typedef bool (*OmfTexcoord32Source)(void *object, float uv[2]);
typedef bool (*OmfTexcoord64Source)(void *object, double uv[2]);
typedef bool (*OmfBoundaryFloat32Source)(void *object, float *value, bool *inclusive);
typedef bool (*OmfBoundaryFloat64Source)(void *object, double *value, bool *inclusive);
typedef bool (*OmfBoundaryInt64Source)(void *object, int64_t *value, bool *inclusive);
typedef bool (*OmfRegularSubblockSource)(void *object, uint32_t parent_index[3], uint32_t corners[6]);
typedef bool (*OmfFreeformSubblock32Source)(void *object, uint32_t parent_index[3], float corners[6]);
typedef bool (*OmfFreeformSubblock64Source)(void *object, uint32_t parent_index[3], double corners[6]);
typedef bool (*OmfNumberFloat32Source)(void *object, float *number, bool *is_null);
typedef bool (*OmfNumberFloat64Source)(void *object, double *number, bool *is_null);
typedef bool (*OmfNumberInt64Source)(void *object, int64_t *number, bool *is_null);
typedef bool (*OmfIndexSource)(void *object, uint32_t *index, bool *is_null);
typedef bool (*OmfVector32x2Source)(void *object, float vector[2], bool *is_null);
typedef bool (*OmfVector32x3Source)(void *object, float vector[3], bool *is_null);
typedef bool (*OmfVector64x2Source)(void *object, double vector[2], bool *is_null);
typedef bool (*OmfVector64x3Source)(void *object, double vector[3], bool *is_null);
typedef bool (*OmfTextSource)(void *object, const char **string, size_t *len);
typedef bool (*OmfBooleanSource)(void *object, bool *boolean, bool *is_null);
typedef bool (*OmfColorSource)(void *object, uint8_t rgba[4], bool *is_null);
```

Callback function types for the various array sources. 

When called, the function should either fill in the output arguments with the next item and return true,
or return false if no more items are available.
The function will be called repeatedly until it returns false.
The common `object` argument provides the object that was passed to the writer function,
use it to store your iterator state.
For nullable types `*is_null` is initialized to false so you can ignore it if all your values are valid.

For `OmfNameSource` and `OmfTextSource` set `len` if you know the string length,
or leave it untouched for nul-terminated strings.
`OmfNameSource` will treat a null string as empty, while `OmfTextSource` will store the null value.
For both, the string buffer only needs to be valid until the next call to your function.

For dates you should use `OmfNumberInt64Source` and output the number of days since the epoch.
For date-time use the same but output the number of microseconds since the epoch in UTC.

If you're implementing these functions in C++ then they **must not** raise exceptions.

## OmfWriter

### Methods

#### omf_writer_array_…_iter

```c
const OmfArray *omf_writer_array_scalars32_iter            (OmfWriter *writer,
                                                            OmfScalar32Source source,
                                                            void *object);
const OmfArray *omf_writer_array_scalars64_iter            (OmfWriter *writer,
                                                            OmfScalar64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_vertices32_iter           (OmfWriter *writer,
                                                            OmfVertex32Source source,
                                                            void *object);
const OmfArray *omf_writer_array_vertices64_iter           (OmfWriter *writer,
                                                            OmfVertex64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_segments_iter             (OmfWriter *writer,
                                                            OmfSegmentSource source,
                                                            void *object);
const OmfArray *omf_writer_array_triangles_iter            (OmfWriter *writer,
                                                            OmfTriangleSource source,
                                                            void *object);
const OmfArray *omf_writer_array_names_iter                (OmfWriter *writer,
                                                            OmfNameSource source,
                                                            void *object);
const OmfArray *omf_writer_array_gradient_iter             (OmfWriter *writer,
                                                            OmfGradientSource source,
                                                            void *object);
const OmfArray *omf_writer_array_texcoords32_iter          (OmfWriter *writer,
                                                            OmfTexcoord32Source source,
                                                            void *object);
const OmfArray *omf_writer_array_texcoords64_iter          (OmfWriter *writer,
                                                            OmfTexcoord64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_boundaries_float32_iter   (OmfWriter *writer,
                                                             OmfBoundaryFloat32Source source,
                                                             void *object);
const OmfArray *omf_writer_array_boundaries_float64_iter   (OmfWriter *writer,
                                                            OmfBoundaryFloat64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_boundaries_int64_iter     (OmfWriter *writer,
                                                            OmfBoundaryInt64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_boundaries_date_iter      (OmfWriter *writer,
                                                            OmfBoundaryInt64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_boundaries_date_time_iter (OmfWriter *writer,
                                                            OmfBoundaryInt64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_regular_subblocks_iter    (OmfWriter *writer,
                                                            OmfRegularSubblockSource source,
                                                            void *object);
const OmfArray *omf_writer_array_freeform_subblocks32_iter (OmfWriter *writer,
                                                            OmfFreeformSubblock32Source source,
                                                            void *object);
const OmfArray *omf_writer_array_freeform_subblocks64_iter (OmfWriter *writer,
                                                            OmfFreeformSubblock64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_numbers_float32_iter      (OmfWriter *writer,
                                                            OmfNumberFloat32Source source,
                                                            void *object);
const OmfArray *omf_writer_array_numbers_float64_iter      (OmfWriter *writer,
                                                            OmfNumberFloat64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_numbers_int64_iter        (OmfWriter *writer,
                                                            OmfNumberInt64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_numbers_date_iter         (OmfWriter *writer,
                                                            OmfNumberInt64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_numbers_date_time_iter    (OmfWriter *writer,
                                                            OmfNumberInt64Source source,
                                                            void *object);
const OmfArray *omf_writer_array_indices_iter              (OmfWriter *writer,
                                                            OmfIndexSource source,
                                                            void *object);
const OmfArray *omf_writer_array_vectors32x2_iter          (OmfWriter *writer,
                                                            OmfVector32x2Source source,
                                                            void *object);
const OmfArray *omf_writer_array_vectors32x3_iter          (OmfWriter *writer,
                                                            OmfVector32x3Source source,
                                                            void *object);
const OmfArray *omf_writer_array_vectors64x2_iter          (OmfWriter *writer,
                                                            OmfVector64x2Source source,
                                                            void *object);
const OmfArray *omf_writer_array_vectors64x3_iter          (OmfWriter *writer,
                                                            OmfVector64x3Source source,
                                                            void *object);
const OmfArray *omf_writer_array_text_iter                 (OmfWriter *writer,
                                                            OmfTextSource source,
                                                            void *object);
const OmfArray *omf_writer_array_booleans_iter             (OmfWriter *writer,
                                                            OmfBooleanSource source,
                                                            void *object);
const OmfArray *omf_writer_array_colors_iter               (OmfWriter *writer,
                                                            OmfColorSource source,
                                                            void *object);
```

These functions add an array by repeatedly calling of the callback functions defined above.
