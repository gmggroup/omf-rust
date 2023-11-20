## OmfComposite

```c
typedef struct {
    size_t n_elements;
    const OmfElement *elements;
} OmfComposite;
```

A container for a set of related sub-elements.

### Attribute Locations

- `OMF_LOCATION_ELEMENTS` to apply one value to each sub-element.


### Fields

n_elements: `size_t`
: Number of sub-elements.

elements: [`const OmfElement *`](../element.md#OmfElement)
: Pointer to an array of `n_elements` sub-elements.

### Methods

#### omf_composite_init

```c
OmfComposite omf_composite_init(void);
```

Initializes or resets a composite struct.
