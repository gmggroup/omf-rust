## OmfSurface

```c
typedef struct {
    double origin[3];
    const OmfArray *vertices;
    const OmfArray *triangles;
} OmfSurface;
```

A triangulated surface.

### Attribute Locations

- `OMF_LOCATION_VERTICES` for per-vertex data.
- `OMF_LOCATION_PRIMITIVES` for per-triangle data.

### Fields

origin: `double[3]`
: An offset to apply to all vertices, along with the [project](../project.md) origin.

vertices: [`const OmfArray *`](../arrays.md#omfarray)
: Vertex array.

segments: [`const OmfArray *`](../arrays.md#omfarray)
: Triangle array.
Each row contains the three vertex indices of one triangle,
ordered counter-clockwise around an outward-pointing normal vector.
Values must be less than the length of `vertices`.

### Methods

#### omf_surface_init

```c
OmfSurface omf_surface_init(const OmfArray *vertices, const OmfArray *triangles);
```

Initializes or resets a surface struct.
