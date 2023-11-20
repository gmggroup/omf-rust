# OMF JSON Schema

The JSON index within the OMF file is specified using a [JSON Schema](http://json-schema.org/).
Below are links to all of the objects within the schema, with their documentation.

- [Project](schema/Project.md) is the root object.
    - [Element](schema/Element.md) stores the name, geometry, attributes, and other details of one object.
        - [Geometry](schema/Geometry.md) picks from points, lines, surface, etc.
            - [PointSet](schema/PointSet.md) describes a set of points.
            - [LineSet](schema/LineSet.md) describes a set of straight line segments.
            - [Surface](schema/Surface.md) describes a triangulated surface.
            - [GridSurface](schema/GridSurface.md) describes a 2D grid surface.
                - [Grid2](schema/Grid2.md) defines the grid, regular or tensor.
                - [Orient2](schema/Orient2.md) gives the position and orientation.
            - [BlockModel](schema/BlockModel.md) describes a block model.
                - [Grid3](schema/Grid3.md) defines the grid, regular or tensor.
                - [Orient3](schema/Orient3.md) gives the position and orientation.
                - [Subblocks](schema/Subblocks.md) describes sub-blocks within each parent block.
                    - [SubblockMode](schema/SubblockMode.md) further restricts sub-blocks to octree or fully sub-blocked.
            - [Composite](schema/Composite.md) contains other elements under a single name.
    - [Attribute](schema/Attribute.md) stores data that is attached to an element.
        - [Location](schema/Location.md) says where the attribute is attached.
        - [AttributeData](schema/AttributeData.md) picks from several type of data.
            - [NumberColormap](schema/NumberColormap.md) maps from numbers to colors.
            - [NumberColormapRange](schema/NumberColormapRange.md) holds a number range for a colormap.
- [Array](schema/Array.md) points to an array or image file in the archive.
