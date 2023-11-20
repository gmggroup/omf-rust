# Example uses of the C API

Examples in this folder, best read in order:

- [`pyramid.c`](./pyramid.c) writes a file containing a small surface and line-set of a square
  pyramid, then reads it back and prints everything.
- [`metadata.c`](./metadata.c) stores and retrieves metadata, including nested structures.
- [`geometries.c`](./geometries.c) stores and retrieves the remaining element geometries:
  point set, grid surface, block models, and composite.
- [`attributes.c`](./attributes.c) puts different types of attributes on a cube surface, then
  reads back and prints a few of them.
- [`textures.c`](./textures.c) creates mapped and projected textures from a pre-existing image
  file and reads them back as pixels.
