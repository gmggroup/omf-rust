## OmfElement

```c
typedef struct {
    const char *name;
    const char *description;
    bool color_set;
    uint8_t color[3];
    double opacity;
    size_t n_metadata;
    const OmfValue *metadata;
    size_t n_attributes;
    const OmfAttribute *attributes;
    const OmfPointSet *point_set;
    const OmfLineSet *line_set;
    const OmfSurface *surface;
    const OmfGridSurface *grid_surface;
    const OmfBlockModel *block_model;
    const OmfComposite *composite;
} OmfElement;
```

Describes an object or shape within an OMF file, and links to attributes that attach data to it.

### Fields

name: `const char *`
: Element name, which should be unique.

description: `const char *`
: Optional element description or comments.

color_set: `bool `
: Whether `color` should be used.

color: `uint8_t[3] `
: Optional solid RGB color of the element.

opacity: `double `
: Element opacity, from 0.0 for transparent to 1.0 for opaque.

n_metadata: `size_t`
: Number of metadata items.

metadata: [`const OmfValue *`](metadata.md#omfvalue)
: Pointer to an array of `n_metadata` metadata items, forming a set of key/value pairs.

n_attributes: `size_t `
: Number of attributes.

attributes: [`const OmfAttribute *`](attribute.md#omfattribute)
: Pointer to an array of `n_attributes` attributes on this element.

point_set: [`const OmfPointSet *`](geometry/pointset.md)
line_set: [`const OmfLineSet *`](geometry/lineset.md)
surface: [`const OmfSurface *`](geometry/surface.md)
grid_surface: [`const OmfGridSurface *`](geometry/gridsurface.md)
block_model: [`const OmfBlockModel *`](geometry/blockmodel.md)
composite: [`const OmfComposite *`](geometry/composite.md)
: Exactly one of these must be non-null. This defines the type of geometry the element has
and contains the details of it.


### Methods

#### omf_element_init

```c
struct OmfElement omf_element_init(const char *name);
```

Initializes or resets an element struct.
