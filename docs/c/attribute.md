# Attribute

## OmfLocation

```c
typedef enum {
    OMF_LOCATION_VERTICES,
    OMF_LOCATION_PRIMITIVES,
    OMF_LOCATION_SUBBLOCKS,
    OMF_LOCATION_ELEMENTS,
    OMF_LOCATION_PROJECTED,
    OMF_LOCATION_CATEGORIES,
} OmfLocation;
```

Defines what part of the geometry an attribute is applied to.
The documentation of each geometry includes what locations are valid for it.

### Options

OMF_LOCATION_VERTICES
: The attribute contains one value for each point, vertex, or block corner.

OMF_LOCATION_PRIMITIVES
: The attribute contains one value for each line segment, triangle, or block.
For sub-blocked block models that means parent blocks.

OMF_LOCATION_SUBBLOCKS
: The attribute contains one value for each sub-block in a block model.

OMF_LOCATION_ELEMENTS
: The attribute contains one value for each sub-element in composite.

OMF_LOCATION_PROJECTED
: Used for projected textures.
The texture is projected onto the element.

OMF_LOCATION_CATEGORIES
: Used for category sub-attributes.
The attribute contains one value for each category.


## OmfAttribute

```c
typedef struct {
    const char *name;
    const char *description;
    const char *units;
    size_t n_metadata;
    const OmfValue *metadata;
    OmfLocation location;
    const OmfArray *boolean_data;
    const OmfArray *vector_data;
    const OmfArray *text_data;
    const OmfArray *color_data;
    const OmfNumberData *number_data;
    const OmfCategoryData *category_data;
    const OmfMappedTexture *mapped_texture_data;
    const OmfProjectedTexture *projected_texture_data;
} OmfAttribute;
```

Contains attribute data and defines how it is applied to an element.
Exactly one data pointer must be non-null, defining the type of the attribute.

### Fields

name: `const char *`
: Attribute name, which should be unique within the containing element.

description: `const char *`
: Optional attribute description or comments.

units: `const char *`
: Optional attribute units, if applicable.
OMF does not currently attempt to standardize the strings you can use here, but our recommendations are:

    - Use full names, so "kilometers" rather than "km".
      The abbreviations for non-metric units aren't consistent and complex units can be confusing.
    - Use plurals, so "feet" rather than "foot".
    - Avoid ambiguity, so "long tons" rather than just "tons".
    - Accept American and British spellings, so "meter" and "metre" are the same.

metadata: [`const OmfValue *`](metadata.md#omfvalue)
: Pointer to an array of `n_metadata` metadata items, forming a set of key/value pairs.

n_attributes: `size_t `
: Number of attributes.

location: [`OmfLocation`](#omflocation)
: Defines where on the containing element this attribute is attached.

boolean_data: [`const OmfArray *`](arrays.md#omfarray)
: Boolean array containing true/false/null data.
Applications that don't support three-valued logic may treat null as false.

vector_data: [`const OmfArray *`](arrays.md#omfarray)
: Vector array. Items are nullable.

text_data: [`const OmfArray *`](arrays.md#omfarray)
: Text array. Items are nullable.

color_data: [`const OmfArray *`](arrays.md#omfarray)
: Color array. Items are nullable.

number_data: [`const OmfNumberData *`](#omfnumberdata)
: Pointer to a number-data struct including the array and optional color-map.

category_data: [`const OmfCategoryData *`](#omfcategorydata)
: Pointer to a category-data struct including the index array, plus category names,
optional colors, and sub-attributes.

mapped_texture_data: [`const OmfMappedTexture *`](#omfmappedtexture)
: Pointer to a mapped texture struct.

projected_texture_data: [`const OmfProjectedTexture *`](#omfprojectedtexture)
: Pointer to a projected texture struct.


### Methods

#### omf_attribute_init

```c
OmfAttribute omf_attribute_init(const char *name, OmfLocation location);
```

Initializes or resets an attribute struct.


## OmfNumberData

```c
typedef struct {
    const OmfArray *values;
    const OmfContinuousColormap *continuous_colormap;
    const OmfDiscreteColormap *discrete_colormap;
} OmfNumberData;
```

### Fields

values: `const OmfArray *`
: Number array. Can have 32- or 64-bit floating-point, 64-bit signed integer,
date, or date-time type.

continuous_colormap: `const OmfContinuousColormap *`
: Optional continuous colormap.
Only one of `continuous_colormap` and `discrete_colormap` may be non-null,
or both may be null.

discrete_colormap: `const OmfDiscreteColormap *`
: Optional discrete colormap.

### Methods

#### omf_number_data_init

```c
OmfNumberData omf_number_data_init(void);
```

Initializes or resets a number attribute data struct.


## OmfCategoryData

```c
typedef struct {
    const OmfArray *values;
    const OmfArray *names;
    const OmfArray *colors;
    const OmfAttribute *attributes;
    size_t n_attributes;
} OmfCategoryData;
```

Describes a category attribute.

### Fields

values: `const OmfArray *`
: Index array into `names`, `colors`, and other attributes. Indices are nullable.

names: `const OmfArray *`
: Name array for category names or labels.

colors: `const OmfArray *`
: Optional gradient array for category colors.
If non-null, must be the same length as `names`.

attributes: `const OmfAttribute *`
: Pointer to an array of `n_attributes` attribute structures.
Each must use `OMF_LOCATION_CATEGORIES` and be the same length as `names`.
This can be used to add extra details to a category, such as density on a rock-type attribute.

n_attributes: `size_t`
: The number of sub-attributes.


### Methods

#### omf_category_data_init

```c
OmfCategoryData omf_category_data_init(void);
```

Initializes or resets a category attribute data struct.


## OmfMappedTexture

```c
typedef struct {
    const OmfArray *image;
    const OmfArray *texcoords;
} OmfMappedTexture;
```

A texture applied with [UV mapping](https://en.wikipedia.org/wiki/UV_mapping).
Typically applied to surface vertices; applications may ignore other locations.

### Fields

image: `const OmfArray *`
: Image array containing the texture image.

texcoords: [`const OmfArray *`](arrays.md#omfarray)
: Texture coordinate array,
Values outside of the range 0–1 will cause the texture to wrap.

### Methods

#### omf_mapped_texture_init

```c
OmfMappedTexture omf_mapped_texture_init(const OmfArray *image,
                                         const OmfArray *texcoords);
```

Initializes or resets a mapped texture struct.


## OmfProjectedTexture

```c
typedef struct {
    const OmfArray *image;
    OmfOrient2 orient;
    double width;
    double height;
} OmfProjectedTexture;
```

A texture that is orthographically projected through space.
Use this for maps and section images.
The fields define a rectangle in space and the texture is projected in both directions along its normal.
Typically applied to surface vertices; applications may ignore other locations.

### Fields

image: `const OmfArray *`
: Image array containing the texture image.

orient: `OmfOrient2`
: The position and orientation of the texture rectangle in space.

width: `double`
: The width of the texture rectangle in space.

height: `double`
: The height of the texture rectangle in space.

### Methods

#### omf_projected_texture_init

```c
OmfProjectedTexture omf_projected_texture_init(const OmfArray *image);
```

Initializes or resets a projected texture struct.
