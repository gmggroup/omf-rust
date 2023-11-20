# Line Set

## OmfLineSet

```c
typedef struct {
    double origin[3];
    const OmfArray *vertices;
    const OmfArray *segments;
} OmfLineSet;
```

A set of straight line segments.

### Attribute Locations

- `OMF_LOCATION_VERTICES` for per-vertex data.
- `OMF_LOCATION_PRIMITIVES` for per-segment data.

### Fields

origin: `double[3]`
: An offset to apply to all vertices, along with the [project](../project.md) origin.

vertices: [`const OmfArray *`](../arrays.md#omfarray)
: Vertex array.

segments: [`const OmfArray *`](../arrays.md#omfarray)
: Segment array. Values must be less than the length of `vertices`.

### Methods

#### omf_line_set_init

```c
OmfLineSet omf_line_set_init(const OmfArray *vertices, const OmfArray *segments);
```

Initializes or resets a line-set struct.
