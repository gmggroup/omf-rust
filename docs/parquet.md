# Parquet Schemas

OMF uses several different types of arrays.
Most types accept more than one Parquet schema, allowing flexibility on the types used.

The sections below describe each array type and give the accepted schema for it.

Take care when using 32-bit floating point values for vertex locations.
They will make file sizes smaller, but it's easy to lose precision.
When converting 64-bit floating-point to 32-bit you should calculate and subtract the
center point from all vertex locations,
then store that offset in the element origin field.

Using 8-bit or 16-bit unsigned integers won't affect the file size,
because Parquet already compresses those efficiently.
It will still help applications loading the file,
letting them use the smaller type when loading the data without needing to check maximum values first.

## Scalar

Floating-point scalar values.
```text
--8<-- "docs/parquet/Scalar.txt"
```

## Vertex

Vertex locations in 3D. Add the project and element origins.
```text
--8<-- "docs/parquet/Vertex.txt"
```

## Segment

Line segments as indices into a vertex array.
```text
--8<-- "docs/parquet/Segment.txt"
```

## Triangle

Triangles as indices into a vertex array.
Triangle winding should be counter-clockwise around an outward-pointing normal
```text
--8<-- "docs/parquet/Triangle.txt"
```

## Name

Non-nullable category names.
These should be unique and non-empty.
```text
--8<-- "docs/parquet/Name.txt"
```

## Gradient

Non-nullable colormap or category colors.
```text
--8<-- "docs/parquet/Gradient.txt"
```

## Texcoord

UV texture coordinates.
Values outside [0, 1] should cause the texture to wrap.
```text
--8<-- "docs/parquet/Texcoord.txt"
```

## Boundary

Discrete color-map boundaries.
If the `inclusive` column is true then the boundary is less than or equal to the value,
otherwise it is less the value.
```text
--8<-- "docs/parquet/Boundary.txt"
```

## RegularSubblock

Parent indices and corners of regular sub-blocks.
These sub-blocks lie on a regular grid within their parent block and defined by integer indices on
that grid.
```text
--8<-- "docs/parquet/RegularSubblock.txt"
```

## FreeformSubblock

Parent indices and corners of free-form sub-blocks.
These sub-blocks can be anywhere within their parent block and are defined relative to it.
```text
--8<-- "docs/parquet/FreeformSubblock.txt"
```

## Number

Nullable number values, which can be floating-point, signed integer, date, or date-time.
Date-time must be in UTC with microsecond precision.
```text
--8<-- "docs/parquet/Number.txt"
```

## Index

Nullable category index values.
```text
--8<-- "docs/parquet/Index.txt"
```

## Vector

Nullable 2D or 3D vectors.
```text
--8<-- "docs/parquet/Vector.txt"
```

## Text

Nullable text.
Some application may treat null as equivalent to an empty string.
```text
--8<-- "docs/parquet/Text.txt"
```

## Boolean

Nullable booleans.
The values are optional, allowing them to be true, false, or null.
Applications that don't support
[three-valued logic](https://en.wikipedia.org/wiki/Three-valued_logic) may treat null as false.
```text
--8<-- "docs/parquet/Boolean.txt"
```

## Color

Nullable colors, in 8-bit RGB or RGBA.
Omitting the alpha column out is equivalent to setting all alpha values to 255.
```text
--8<-- "docs/parquet/Color.txt"
```
