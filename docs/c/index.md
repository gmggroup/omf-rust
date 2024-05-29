# OMF C API

## Examples

C examples, from simplest to most complex:

- [`pyramid.c`](./examples/pyramid.md)
writes a file containing a small surface and line-set of a square pyramid,
then reads it back and prints everything.
- [`metadata.c`](./examples/metadata.md)
stores and retrieves metadata, including nested structures.
- [`geometries.c`](./examples/geometries.md)
stores and retrieves the remaining element geometries:
point set, grid surface, block models, and composite.
- [`attributes.c`](./examples/attributes.md)
puts different types of attributes on a cube surface,
then reads back and prints a few of them.
- [`textures.c`](./examples/textures.md)
creates mapped and projected textures from a pre-existing image and reads it back as pixels.

## By Section

- [Errors](errors.md)
- [Metadata](metadata.md)
- [Arrays](arrays.md)
- [Images](images.md)
- [Project](project.md)
- [Element](element.md)
- Geometries:
    - [Point Set](geometry/pointset.md)
    - [Line Set](geometry/lineset.md)
    - [Surface](geometry/surface.md)
    - [Grid Surface](geometry/gridsurface.md)
    - [Composite](geometry/composite.md)
    - [Block Model](geometry/blockmodel.md)
- [Attribute](attribute.md)
- [Color maps](colormap.md)
- [Grid Position and Orientation](grids.md)
- [Reader](reader.md)
    - [Reader Iterators](reader_iterators.md)
- [Writer](writer.md)
    - [Writer Iterators](writer_iterators.md)
- [OMF v1 Conversion](omf1.md)
