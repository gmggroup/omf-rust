# Point Set

## OmfPointSet

```c
typedef struct {
    double origin[3];
    const OmfArray *vertices;
} OmfPointSet;
```

A set of point locations in space.

### Attribute Locations

- `OMF_LOCATION_VERTICES` for per-point data.

### Fields

origin: `double[3]`
: An offset to apply to all points, along with the [project](../project.md) origin.

vertices: [`const OmfArray *`](../arrays.md#omfarray)
: Vertex array.


### Methods

#### omf_point_set_init

```c
OmfPointSet omf_point_set_init(const OmfArray *vertices);
```

Initializes or resets a point-set struct.
