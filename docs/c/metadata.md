# Metadata

## OmfValueType

Stores arbitrary metadata for OMF project, elements, and attributes.
Metadata values come in a variety of types and can be nested.

```c
typedef enum {
    OMF_VALUE_TYPE_NULL,
    OMF_VALUE_TYPE_BOOLEAN,
    OMF_VALUE_TYPE_NUMBER,
    OMF_VALUE_TYPE_STRING,
    OMF_VALUE_TYPE_ARRAY,
    OMF_VALUE_TYPE_OBJECT,
} OmfValueType;
```

## OmfValue

```c
typedef struct OmfValue {
    const char *name;
    OmfValueType type;
    bool boolean;
    double number;
    const char *string;
    const struct OmfValue *values;
    size_t n_values;
} OmfValue;
```

### Fields

name: `const char*`
: The value name. Ignored for values inside metadata arrays.

type: `OmfValueType`
: The value type, which defines which is the following fields is to be used.

boolean: `bool`
: The boolean value if `type` is `OMF_VALUE_TYPE_BOOLEAN`.

number: `double`
: The number value if `type` is `OMF_VALUE_TYPE_NUMBER`.

string: `const char*`
: The string value if `type` is `OMF_VALUE_TYPE_STRING`, in UTF-8 encoding.

values: `const struct OmfValue*`
: The array of sub-values.
If `type` is `OMF_VALUE_TYPE_ARRAY` then the values in order form an array.
If `type` is  `OMF_VALUE_TYPE_OBJECT` then the value names should be used to form a map of key/value pairs.
Otherwise this will be null and `n_values` will be zero.

n_values: `size_t`
: The number of sub-values.
