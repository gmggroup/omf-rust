# Arrays

Types that describe data arrays within the OMF file.

## OmfArrayType

Contains the type of an array.

```c
typedef enum { ... } OmfArrayType;
```

### Options

OMF_ARRAY_TYPE_IMAGE
: An array that stores a single image.

OMF_ARRAY_TYPE_SCALARS32
: Scalar data in 32-bit floating-point.

OMF_ARRAY_TYPE_SCALARS64
: Scalar data in 64-bit floating-point.

OMF_ARRAY_TYPE_VERTICES32
: Vertex positions in 32-bit floating-point.

OMF_ARRAY_TYPE_VERTICES64
: Vertex positions in 64-bit floating-point.

OMF_ARRAY_TYPE_SEGMENTS
: Segments as pairs of indices into a vertex array.

OMF_ARRAY_TYPE_TRIANGLES
: Triangles as triples of indices into a vertex array.
Winding is counter-clockwise around an outward-pointing normal.

OMF_ARRAY_TYPE_NAMES
: Category names.

OMF_ARRAY_TYPE_GRADIENT
: Category or colormap colors.

OMF_ARRAY_TYPE_TEXCOORDS32
: UV texture coordinates in 32-bit floating-point.

OMF_ARRAY_TYPE_TEXCOORDS64
: UV texture coordinates in 64-bit floating-point.

OMF_ARRAY_TYPE_BOUNDARIES_FLOAT32
: Discrete colormap boundaries, storing a value and inclusive flag, in 32-bit floating-point.

OMF_ARRAY_TYPE_BOUNDARIES_FLOAT64
: Discrete colormap boundaries, storing a value and inclusive flag, in 64-bit floating-point.

OMF_ARRAY_TYPE_BOUNDARIES_INT64
: Discrete colormap boundaries, storing a value and inclusive flag, in 64-bit integer.

OMF_ARRAY_TYPE_BOUNDARIES_DATE
: Discrete colormap boundaries, storing a value and inclusive flag, in days since the epoch.

OMF_ARRAY_TYPE_BOUNDARIES_DATE_TIME
: Discrete colormap boundaries, storing a value and inclusive flag, in microseconds since the epoch.

OMF_ARRAY_TYPE_REGULAR_SUBBLOCKS
: Regular sub-block parent indices and min/max corners.

OMF_ARRAY_TYPE_FREEFORM_SUBBLOCKS32
: Free-form sub-block parent indices, and min/max corners in 32-bit floating-point.

OMF_ARRAY_TYPE_FREEFORM_SUBBLOCKS64
: Free-form sub-block parent indices, and min/max corners in 64-bit floating-point.

OMF_ARRAY_TYPE_NUMBERS_FLOAT32
: Nullable number attribute values, in 32-bit floating-point.

OMF_ARRAY_TYPE_NUMBERS_FLOAT64
: Nullable number attribute values, in 64-bit floating-point.

OMF_ARRAY_TYPE_NUMBERS_INT64
: Nullable number attribute values, in 64-bit integer.

OMF_ARRAY_TYPE_NUMBERS_DATE
: Nullable number attribute values, in days since the epoch.

OMF_ARRAY_TYPE_NUMBERS_DATE_TIME
: Nullable number attribute values, in microseconds since the epoch.

OMF_ARRAY_TYPE_INDICES
: Nullable index values for category attributes.

OMF_ARRAY_TYPE_VECTORS32X2
: Nullable 2D vectors in 32-bit floating-point.

OMF_ARRAY_TYPE_VECTORS64X2
: Nullable 2D vectors in 64-bit floating-point.

OMF_ARRAY_TYPE_VECTORS32X3
: Nullable 3D vectors in 32-bit floating-point.

OMF_ARRAY_TYPE_VECTORS64X3
: Nullable 3D vectors in 64-bit floating-point.

OMF_ARRAY_TYPE_TEXT
: Nullable text in UTF-8 encoding and nul-terminated.
Applications may treat empty an null strings as the same thing.

OMF_ARRAY_TYPE_BOOLEANS
: Nullable boolean values: true, false, or null.
Applications that don't support three-valued logic may treat null as false.

OMF_ARRAY_TYPE_COLORS
: Nullable RGBA colors, 8-bits per channel.


## OmfArray

References an array in the OMF file.
A single type is used to store references to all the different array types.
See [`OmfReader`](reader.md) and [`OmfWriter`](writer.md) for the functions that create and consume arrays.

```c
typedef struct { /* private fields */ } OmfArray;
```

## OmfArrayInfo

```c
typedef struct {
    OmfArrayType array_type;
    uint64_t item_count;
    uint64_t compressed_size;
} OmfArrayInfo;
```

### Fields

array_type: [`OmfArrayType`](#OmfArrayType)
: The type of data that the array stores.

item_count: `uint64_t`
: The number of items in the array.

compressed_size: `uint64_t`
: The compressed size of the array inside the Zip archive.
