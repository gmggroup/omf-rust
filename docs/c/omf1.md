# OMF v1 Conversion

This library can convert existing OMF v1 files to OMF v2.
This is a standalone process that reads the OMF v1 file and writes a new OMF v2 file.

## Conversion details

There are a few parts of OMF1 that don't map directly to OMF2.

### Elements

- The `date_created` and `date_modified` fields are moved into the metadata.
- The `subtype` field on point-sets and line-sets is moved into the metadata.
  On other elements, where it only had one valid value, it is discarded.
- Line-sets and surfaces with invalid vertex indices will cause conversion to fail.
- Line-sets and surfaces with more than 4,294,967,295 vertices will cause conversion to fail.

### Data to Attributes

- Scalar data becomes a number attribute, preserving the float/int type of the array.
- In number data, NaN becomes null.
- In 2D or 3D vector data, if any component is NaN the vector becomes null.
- In string data, empty strings become nulls.
  OMF2 supports both null and empty string so we can only guess which was intended.
- In date-time data, empty strings become null.
- Date-times outside the range of approximately ±262,000 years CE will cause conversion to fail.

### Mapped Data to Category Attribute

The exact layout of mapped data from OMF v1 can't be stored in OMF v2.
It is transformed to a category attribute by following these rules:

- Indices equal to minus one become null.
- Indices outside the range 0 to 4,294,967,295 will cause conversion to fail.
- The most unique, least empty, and shortest string legend becomes the category names,
  padded with empty strings if necessary.
- The most unique and least empty color legend becomes the category colors, padded with
  gray if necessary.
- Other legends become extra attributes, padded with nulls if necessary.

## omf_omf1_detect

```c
bool omf_omf1_detect(const char *path);
```

Returns true if the file at the given path looks more like OMF v1 than OMF v2.
This is a very quick check and doesn't guarantee that the file will load successfully.

Returns false on error, or if file open or read fails; call `omf_error()` to tell the difference.

## OmfOmf1Converter

```c
typedef struct { /* private fields */ } Omf1Converter;
```

The object that handles OMF v1 conversion. The general usage pattern is:

1. Create the object.
1. Set parameters.
1. Convert one or more files.
1. Free the object.

### Methods

#### omf_omf1_converter_new

```c
OmfOmf1Converter *omf_omf1_converter_new(void);
```

Creates a new OMF v1 converter with default parameters.
Returns null on error.
Pass the returned pointer to `omf_omf1_converter_free` when you're finished with it.

#### omf_omf1_converter_free

```c
bool omf_omf1_converter_free(OmfOmf1Converter *converter);
```

Frees a converter returned by `omf_omf1_converter_new`.
Returns false on error.

#### omf_omf1_converter_compression

```c
int32_t omf_omf1_converter_compression(struct OmfOmf1Converter *converter);
```

Returns the current compression level of the converter, or -1 on error.

#### omf_omf1_converter_set_compression

```c
bool omf_omf1_converter_set_compression(struct OmfOmf1Converter *converter, int32_t compression);
```

Sets the compression to use when writing the OMF v2 file.
Pass an integer between 1 for fastest and 9 for most compressed, or -1 to use the default.

Returns false on error.

#### omf_omf1_converter_limits

```c
struct OmfLimits omf_omf1_converter_limits(struct OmfOmf1Converter *converter);
```

Returns the current limits to be used when reading the OMF v1 file.

#### omf_omf1_converter_set_limits

```c
bool omf_omf1_converter_set_limits(struct OmfOmf1Converter *converter,
                                   const struct OmfLimits *limits);
```

Sets the limits to use when reading the OMF v1 file.
See [`OmfLimits`](./reader.md#omflimits).

Currently only the `json_bytes` field applies to conversion.
All other parts of the file are streamed in and out so the amount of memory used
doesn't depend on the file contents.

Returns false on error.

#### omf_omf1_converter_convert

```c
bool omf_omf1_converter_convert(struct OmfOmf1Converter *converter,
                                const char *input_path,
                                const char *output_path,
                                struct OmfValidation **validation);
```

Runs the actual conversion, reading from `input_path` and writing to `output_path`.
The output file will be created if it doesn't exist, or overwritten if it does.

Returns false on error.
