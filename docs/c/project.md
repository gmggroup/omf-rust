# Project

## OmfProject

```c
typedef struct {
    const char *name;
    const char *description;
    const char *coordinate_reference_system;
    const char *author;
    int64_t date;
    double origin[3];
    size_t n_metadata;
    const OmfValue *metadata;
    size_t n_elements;
    const OmfElement *elements;
} OmfProject;
```

The root object of an OMF file.

### Fields

name: `const char*`
: Project name.

description: `const char *`
: Optional project description or comments.

coordinate_reference_system: `const char *`
: Optional coordinate reference system.

units: `const char *`
: The spacial units used for positions and distances within this project if they aren't defined by
`coordinate_reference_system`.
If no unit is explicitly defined then assume meters.

author: `const char *`
: The name or email address of the creating person.

date: `int64_t`
: The creation date and time, in microseconds since the `1970-01-01T00:00:00Z` epoch.

origin: `double[3]`
: An offset to apply to all locations in the file. 

n_metadata: `size_t`
: Number of metadata items.

metadata: [`const OmfValue *`](metadata.md#omfvalue)
: Pointer to an array of `n_metadata` metadata items, forming a set of key/value pairs.

n_elements: `size_t`
: Number of elements.

elements: [`const OmfElement *`](element.md#omfelement)
: Pointer to an array of `n_elements` elements.


### Methods

#### omf_project_init

```c
OmfProject omf_project_init(const char *name);
```

Initializes or resets a project struct.
