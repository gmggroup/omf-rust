# Grid Surface

## OmfGridSurface

```c
typedef OmfGridSurface {
    OmfOrient2 orient;
    const OmfRegularGrid2 *regular_grid;
    const OmfTensorGrid2 *tensor_grid;
    const OmfArray *heights;
} OmfGridSurface;
```

A 2D grid or surface positioned and oriented in 3D space.

### Attribute Locations

- `OMF_LOCATION_VERTICES` for per-corner data.
- `OMF_LOCATION_PRIMITIVES` for per-cell data.

### Fields

orient: [`OmfOrient2`](#omforient2)
: Contains the position and orientation.

regular_grid: [`const OmfRegularGrid2 *`](../grids.md#omfregulargrid2)
tensor_grid: [`const OmfTensorGrid2 *`](../grids.md#omftensorgrid2)
: Exactly one of these must be non-null, defining a grid with either regular or varied spacing.

heights: [`const OmfArray *`](../arrays.md#omfarray)
: Optional scalar array giving a signed offset from the plane for each grid corner.
If null then the grid is flat.


### Methods

#### omf_grid_surface_init

```c
OmfGridSurface omf_grid_surface_init(void);
```

Initializes or resets a grid surface struct.
