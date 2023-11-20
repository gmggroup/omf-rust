# Errors

## OmfError

```c
typedef struct OmfError {
    int32_t code;
    int32_t detail;
    const char *message;
} OmfError;
```

Stores an error code and message.

### Fields

code: `int32_t`
: An [`OmfStatus`](#OmfStatus) value for the error.

detail: `int32_t`
: If `code` is `OMF_STATUS_IO_ERROR` this is the system error number, otherwise zero.

message: `const char*`
: Human-readable error message in US-English.

### Methods

#### omf_error

```c
OmfError* omf_error(void);
```

Returns and clears the error state of the current thread.
This is the first error that occurred since the last call to `omf_error` or `omf_error_clear`,
or null if no error occurred.

Pass the returned pointer to `omf_error_free` once you're finished with it.

#### omf_error_free

```c
void omf_error_free(OmfError *error);
```

Frees an error pointer returned by `omf_error`. Does nothing if `error` is null.

#### omf_error_clear

```c
void omf_error_clear(void);
```

Clears the error state of the current thread, discarding an recorded error.

#### omf_error_peek

```c
int32_t omf_error_peek(void);
```

Returns the [`OmfStatus`](#OmfStatus) code if an error occurred on the current thread,
or zero if no error.
You can use this to check for errors before calling a handler that will retrieve the full error.


## OmfStatus

```c
typedef enum {
    OMF_STATUS_SUCCESS = 0,
    ...
} OmfStatus;
```

This enum defines the error codes that the library can produce.
Errors will also come with a message string that gives more details.

### Options

OMF_STATUS_SUCCESS = 0
: No error occurred.

OMF_STATUS_PANIC
: Unexpected failure.

OMF_STATUS_INVALID_ARGUMENT
: An invalid argument was passed, such as a pointer being null when that isn't allowed,
or a string not being in UTF-8 encoding.

OMF_STATUS_INVALID_CALL
: A method call was invalid, such as trying to load the project twice from one reader.

OMF_STATUS_OUT_OF_MEMORY
: Failed to allocate enough memory.

OMF_STATUS_IO_ERROR
: File input or output error from the operating system.

OMF_STATUS_NOT_OMF
: The file is not in OMF format.

OMF_STATUS_NEWER_VERSION
: The file version is newer than what this library can load.

OMF_STATUS_PRE_RELEASE
: The file has a pre-release version which can't be loaded.

OMF_STATUS_DESERIALIZATION_FAILED
: JSON deserialization error when reading a file.

OMF_STATUS_SERIALIZATION_FAILED
: JSON serialization error when writing a file.

OMF_STATUS_VALIDATION_FAILED
: The file contains invalid info.

OMF_STATUS_LIMIT_EXCEEDED
: A safety limit was exceeded when reading.

OMF_STATUS_NOT_IMAGE_DATA
: Image bytes are not in PNG or JPEG format.

OMF_STATUS_NOT_PARQUET_DATA
: Array bytes are not in Parquet format.

OMF_STATUS_ARRAY_TYPE_WRONG
: An incorrect array type was used, such as passing triangles where segments are expected.
Note that even types that look superficially similar aren't the same,
such as 3D vectors and vertices different because the vectors are nullable.

OMF_STATUS_BUFFER_LENGTH_WRONG
: Tried to read an array into a buffer with a different length.

OMF_STATUS_INVALID_DATA
: Array data in the file is invalid,
such as having a triangle index that is larger than the number of vertices.

OMF_STATUS_UNSAFE_CAST
: Attempted a cast that would lose data.
Most commonly 64-bit floating-point values to 32-bit, which would lose precision.

OMF_STATUS_ZIP_MEMBER_MISSING
: A file referenced in the JSON index was not found in the Zip archive.

OMF_STATUS_ZIP_ERROR
: The Zip-file sub-system failed.

OMF_STATUS_PARQUET_SCHEMA_MISMATCH
: A Parquet array file did not have the expected schema.

OMF_STATUS_PARQUET_ERROR
: The Parquet sub-system failed.

OMF_STATUS_IMAGE_ERROR
: The image sub-system failed.


## OmfValidation

```c
typedef struct {
    size_t n_messages;
    const char *const *messages;
} OmfValidation;
```

Used when reading or writing a file to return a list of validation errors and warnings.

### Fields

n_messages: `size_t`
: The number of messages.

messages: `const char *const *`
: Array of messages, each a UTF-8 encoded and nul-terminated string in US-English.

### Methods

#### omf_validation_free

```c
bool omf_validation_free(OmfValidation *ptr);
```

Frees an `OmfValidation` pointer. Does nothing if it is null. Returns false on error.
