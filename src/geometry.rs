use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    array::Constraint,
    array_type,
    validate::{Validate, Validator},
    Array, BlockModel, Element, Grid2, Location, Orient2, Vector3,
};

pub(crate) fn zero_origin(v: &Vector3) -> bool {
    *v == [0.0, 0.0, 0.0]
}

/// Selects the type of geometry in an [`Element`](crate::Element) from several options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum Geometry {
    PointSet(PointSet),
    LineSet(LineSet),
    Surface(Surface),
    GridSurface(GridSurface),
    BlockModel(BlockModel),
    Composite(Composite),
}

impl Geometry {
    /// Returns the valid locations for attributes on this geometry.
    pub fn valid_locations(&self) -> &'static [Location] {
        match self {
            Self::PointSet(_) => &[Location::Vertices],
            Self::LineSet(_) => &[Location::Vertices, Location::Primitives],
            Self::Surface(_) => &[Location::Vertices, Location::Primitives],
            Self::GridSurface(_) => &[Location::Vertices, Location::Primitives],
            Self::Composite(_) => &[Location::Elements],
            Self::BlockModel(b) if b.has_subblocks() => {
                &[Location::Subblocks, Location::Primitives]
            }
            Self::BlockModel(_) => &[Location::Primitives],
        }
    }

    /// Returns the length of the given location, if valid.
    pub fn location_len(&self, location: Location) -> Option<u64> {
        if location == Location::Projected {
            Some(0)
        } else {
            match self {
                Self::PointSet(p) => p.location_len(location),
                Self::LineSet(l) => l.location_len(location),
                Self::Surface(s) => s.location_len(location),
                Self::GridSurface(t) => t.location_len(location),
                Self::BlockModel(b) => b.location_len(location),
                Self::Composite(c) => c.location_len(location),
            }
        }
    }

    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::PointSet(_) => "PointSet",
            Self::LineSet(_) => "LineSet",
            Self::Surface(_) => "Surface",
            Self::GridSurface(_) => "GridSurface",
            Self::Composite(_) => "Composite",
            Self::BlockModel(b) if b.has_subblocks() => "BlockModel(sub-blocked)",
            Self::BlockModel(_) => "BlockModel",
        }
    }
}

impl From<PointSet> for Geometry {
    fn from(value: PointSet) -> Self {
        Self::PointSet(value)
    }
}

impl From<LineSet> for Geometry {
    fn from(value: LineSet) -> Self {
        Self::LineSet(value)
    }
}

impl From<Surface> for Geometry {
    fn from(value: Surface) -> Self {
        Self::Surface(value)
    }
}

impl From<GridSurface> for Geometry {
    fn from(value: GridSurface) -> Self {
        Self::GridSurface(value)
    }
}

impl From<BlockModel> for Geometry {
    fn from(value: BlockModel) -> Self {
        Self::BlockModel(value)
    }
}

impl From<Composite> for Geometry {
    fn from(value: Composite) -> Self {
        Self::Composite(value)
    }
}

/// Point set geometry.
///
/// ### Attribute Locations
///
/// - [`Vertices`](crate::Location::Vertices) puts attribute values on the points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PointSet {
    /// Origin of the points relative to the project coordinate reference system.
    #[serde(default, skip_serializing_if = "zero_origin")]
    pub origin: Vector3,
    /// Array with `Vertex` type storing the vertex locations.
    ///
    /// Add `origin` and the [project](crate::Project) origin to get the locations relative
    /// to the project coordinate reference system.
    pub vertices: Array<array_type::Vertex>,
}

impl PointSet {
    pub fn new(vertices: Array<array_type::Vertex>) -> Self {
        Self::with_origin(vertices, Default::default())
    }

    pub fn with_origin(vertices: Array<array_type::Vertex>, origin: Vector3) -> Self {
        Self { origin, vertices }
    }

    pub fn location_len(&self, location: Location) -> Option<u64> {
        match location {
            Location::Vertices => Some(self.vertices.item_count()),
            _ => None,
        }
    }
}

/// A set of line segments.
///
/// ### Attribute Locations
///
/// - [`Vertices`](crate::Location::Vertices) puts attribute values on the vertices.
///
/// - [`Primitives`](crate::Location::Primitives) puts attribute values on the line segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LineSet {
    /// Origin of the lines relative to the project coordinate reference system.
    #[serde(default, skip_serializing_if = "zero_origin")]
    pub origin: Vector3,
    /// Array with `Vertex` type storing the vertex locations.
    ///
    /// Add `origin` and the [project](crate::Project) origin to get the locations relative
    /// to the project coordinate reference system.
    pub vertices: Array<array_type::Vertex>,
    /// Array with `Segment` type storing each segment as a pair of indices into `vertices`.
    pub segments: Array<array_type::Segment>,
}

impl LineSet {
    pub fn new(vertices: Array<array_type::Vertex>, segments: Array<array_type::Segment>) -> Self {
        Self::with_origin(vertices, segments, Default::default())
    }

    pub fn with_origin(
        vertices: Array<array_type::Vertex>,
        segments: Array<array_type::Segment>,
        origin: Vector3,
    ) -> Self {
        Self {
            origin,
            vertices,
            segments,
        }
    }

    pub fn location_len(&self, location: Location) -> Option<u64> {
        match location {
            Location::Vertices => Some(self.vertices.item_count()),
            Location::Primitives => Some(self.segments.item_count()),
            _ => None,
        }
    }
}

/// A surface made up of triangles.
///
/// ### Attribute Locations
///
/// - [`Vertices`](crate::Location::Vertices) puts attribute values on the vertices.
///
/// - [`Primitives`](crate::Location::Primitives) puts attribute values on the triangles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Surface {
    /// Origin of the surface relative to the project coordinate reference system.
    #[serde(default, skip_serializing_if = "zero_origin")]
    pub origin: Vector3,
    /// Array with `Vertex` type storing the vertex locations.
    ///
    /// Add `origin` and the [project](crate::Project) origin to get the locations relative
    /// to the project coordinate reference system.
    pub vertices: Array<array_type::Vertex>,
    /// Array with `Triangle` type storing each triangle as a triple of indices into `vertices`.
    /// Triangle winding should be counter-clockwise around an outward-pointing normal.
    pub triangles: Array<array_type::Triangle>,
}

impl Surface {
    pub fn new(
        vertices: Array<array_type::Vertex>,
        triangles: Array<array_type::Triangle>,
    ) -> Self {
        Self::with_origin(vertices, triangles, Default::default())
    }

    pub fn with_origin(
        vertices: Array<array_type::Vertex>,
        triangles: Array<array_type::Triangle>,
        origin: Vector3,
    ) -> Self {
        Self {
            origin,
            vertices,
            triangles,
        }
    }

    pub fn location_len(&self, location: Location) -> Option<u64> {
        match location {
            Location::Vertices => Some(self.vertices.item_count()),
            Location::Primitives => Some(self.triangles.item_count()),
            _ => None,
        }
    }
}

/// A surface defined by a 2D grid a height on each grid vertex.
///
/// ### Attribute Locations
///
/// - [`Vertices`](crate::Location::Vertices) puts attribute values on the grid vertices.
///
/// - [`Primitives`](crate::Location::Primitives) puts attribute values on the grid cells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GridSurface {
    /// Position and orientation of the surface.
    pub orient: Orient2,
    /// 2D grid definition, which can be regular or tensor.
    pub grid: Grid2,
    /// Array with `Scalar` type storing the offset of each grid vertex from the place.
    /// Heights may be positive or negative. Will be absent from flat 2D grids.
    pub heights: Option<Array<array_type::Scalar>>,
}

impl GridSurface {
    pub fn new(orient: Orient2, grid: Grid2, heights: Option<Array<array_type::Scalar>>) -> Self {
        Self {
            orient,
            grid,
            heights,
        }
    }

    pub fn location_len(&self, location: Location) -> Option<u64> {
        match location {
            Location::Vertices => Some(self.grid.flat_corner_count()),
            Location::Primitives => Some(self.grid.flat_count()),
            _ => None,
        }
    }
}

/// A container for sub-elements.
///
/// ### Attribute Locations
///
/// - [`Elements`](crate::Location::Elements) puts attribute values on elements.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Composite {
    #[serde(default)]
    pub elements: Vec<Element>,
}

impl Composite {
    pub fn new(elements: Vec<Element>) -> Self {
        Self { elements }
    }

    pub fn location_len(&self, location: Location) -> Option<u64> {
        match location {
            Location::Elements => Some(self.elements.len().try_into().expect("usize fits in u64")),
            _ => None,
        }
    }
}

impl Validate for Geometry {
    fn validate_inner(&mut self, val: &mut Validator) {
        let v = val.enter("Geometry");
        match self {
            Self::PointSet(x) => v.obj(x),
            Self::LineSet(x) => v.obj(x),
            Self::Surface(x) => v.obj(x),
            Self::GridSurface(x) => v.obj(x),
            Self::BlockModel(x) => v.obj(x),
            Self::Composite(x) => v.obj(x),
        };
    }
}

impl Validate for PointSet {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("PointSet")
            .finite_seq(self.origin, "origin")
            .array(&mut self.vertices, Constraint::Vertex, "vertices");
    }
}

impl Validate for LineSet {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("LineSet")
            .finite_seq(self.origin, "origin")
            .array(&mut self.vertices, Constraint::Vertex, "vertices")
            .array(
                &mut self.segments,
                Constraint::Segment(self.vertices.item_count()),
                "segments",
            );
    }
}

impl Validate for Surface {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Surface")
            .finite_seq(self.origin, "origin")
            .array(&mut self.vertices, Constraint::Vertex, "vertices")
            .array(
                &mut self.triangles,
                Constraint::Triangle(self.vertices.item_count()),
                "triangles",
            );
    }
}

impl Validate for GridSurface {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("GridSurface")
            .obj(&mut self.orient)
            .obj(&mut self.grid)
            .array_opt(self.heights.as_mut(), Constraint::Scalar, "heights")
            .array_size_opt(
                self.heights.as_ref().map(|h| h.item_count()),
                self.grid.flat_corner_count(),
                "heights",
            );
    }
}

impl Validate for Composite {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Composite").objs(&mut self.elements).unique(
            self.elements.iter().map(|e| &e.name),
            "elements[..]::name",
            false,
        );
    }
}
