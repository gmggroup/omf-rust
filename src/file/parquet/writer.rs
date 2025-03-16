use std::io::{Seek, Write};

use crate::{
    array_type,
    data::{write_checks::*, *},
    error::Error,
    file::zip_container::FileType,
    pqarray::{PqArrayWriter, PqWriteOptions},
    Array, ArrayType,
};

use super::super::{Compression, Writer};

impl From<Compression> for PqWriteOptions {
    fn from(value: Compression) -> Self {
        Self {
            compression_level: value.level(),
            ..Default::default()
        }
    }
}

impl<W: Write + Seek + Send> Writer<W> {
    fn array_writer<'a>(&self) -> PqArrayWriter<'a> {
        PqArrayWriter::new(self.compression().into())
    }

    fn array_write<A: ArrayType>(&mut self, writer: PqArrayWriter) -> Result<Array<A>, Error> {
        let f = self.builder.open(FileType::Parquet)?;
        let name = f.name().to_owned();
        let length = writer.write(f)?;
        Ok(Array::new(name, length))
    }

    /// Write an [`array_type::Scalar`](crate::array_type::Scalar) array.
    ///
    /// Values can be `f32` or `f64`.
    pub fn array_scalars<I, T>(&mut self, data: I) -> Result<Array<array_type::Scalar>, Error>
    where
        I: IntoIterator<Item = T>,
        T: FloatType,
    {
        let mut min = MinimumScalar::new();
        let mut writer = self.array_writer();
        writer.add("scalar", data.into_iter().map(|v| min.visit(v)))?;
        Ok(self.array_write(writer)?.add_write_checks(min.get()))
    }

    /// Write an [`array_type::Vertex`](crate::array_type::Vertex) array.
    pub fn array_vertices<I, T>(&mut self, data: I) -> Result<Array<array_type::Vertex>, Error>
    where
        I: IntoIterator<Item = [T; 3]>,
        T: FloatType,
    {
        let mut writer = self.array_writer();
        writer.add_multiple(&["x", "y", "z"], data)?;
        self.array_write(writer)
    }

    /// Write an [`array_type::Segment`](crate::array_type::Segment) array.
    ///
    /// Values can be `[u8; 2]`, `[u16; 2]`, or `[u32; 2]` and all indices must be less than the
    /// number of vertices.
    pub fn array_segments<I>(&mut self, data: I) -> Result<Array<array_type::Segment>, Error>
    where
        I: IntoIterator<Item = [u32; 2]>,
    {
        let mut max = MaximumIndex::new();
        let mut writer = self.array_writer();
        writer.add_multiple(&["a", "b"], data.into_iter().map(|v| max.visit_array(v)))?;
        Ok(self.array_write(writer)?.add_write_checks(max.get()))
    }

    /// Write an [`array_type::Triangle`](crate::array_type::Triangle) array.
    ///
    /// Values can be `[u8; 3]`, `[u16; 3]`, or `[u32; 3]` and all indices must be less than the
    /// number of vertices.
    pub fn array_triangles<I>(&mut self, data: I) -> Result<Array<array_type::Triangle>, Error>
    where
        I: IntoIterator<Item = [u32; 3]>,
    {
        let mut max = MaximumIndex::new();
        let mut writer = self.array_writer();
        writer.add_multiple(
            &["a", "b", "c"],
            data.into_iter().map(|v| max.visit_array(v)),
        )?;
        Ok(self.array_write(writer)?.add_write_checks(max.get()))
    }

    /// Write an [`array_type::Name`](crate::array_type::Name) array.
    pub fn array_names<I>(&mut self, data: I) -> Result<Array<array_type::Name>, Error>
    where
        I: IntoIterator<Item = String>,
    {
        let mut writer = self.array_writer();
        writer.add("name", data)?;
        self.array_write(writer)
    }

    /// Write an [`array_type::Gradient`](crate::array_type::Gradient) array.
    ///
    /// Values are `[u8; 4]` with channels in RGBA color.
    pub fn array_gradient<I>(&mut self, data: I) -> Result<Array<array_type::Gradient>, Error>
    where
        I: IntoIterator<Item = [u8; 4]>,
    {
        let mut writer = self.array_writer();
        writer.add_multiple(&["r", "g", "b", "a"], data.into_iter())?;
        self.array_write(writer)
    }

    /// Write an [`array_type::Texcoord`](crate::array_type::Texcoord) array.
    ///
    /// Values can be either `[f32; 2]` or `[f64; 2]` containing normalized texture coordinates.
    pub fn array_texcoords<I, T>(&mut self, data: I) -> Result<Array<array_type::Texcoord>, Error>
    where
        I: IntoIterator<Item = [T; 2]>,
        T: FloatType,
    {
        let mut writer = self.array_writer();
        writer.add_multiple(&["u", "v"], data)?;
        self.array_write(writer)
    }

    /// Write an [`array_type::Boundary`](crate::array_type::Boundary) array.
    ///
    /// The boundary value type `T` can be `f64`, `i64`, `chrono::NaiveDate`, or `chrono::DateTime<Utc>`.
    pub fn array_boundaries<I, T>(&mut self, data: I) -> Result<Array<array_type::Boundary>, Error>
    where
        I: IntoIterator<Item = Boundary<T>>,
        T: NumberType,
    {
        let mut increasing = IncreasingBoundary::new();
        let mut writer = self.array_writer();
        writer.add_multiple(
            &["value", "inclusive"],
            data.into_iter()
                .map(|b| (increasing.visit(b.value()), b.is_inclusive())),
        )?;
        Ok(self.array_write(writer)?.add_write_checks(increasing.get()))
    }

    /// Write an [`array_type::RegularSubblock`](crate::array_type::RegularSubblock) array.
    ///
    /// The `parent_indices` and `corners` iterators must be the same length. Each row is
    /// `[parent_i, parent_j, parent_k, min_corner_i, min_corner_j, min_corner_k,
    /// max_corner_i, max_corner_j, max_corner_k]`. The parent and corner indices can
    /// be different types.
    ///
    /// Parent indices can be `[u8; 3]`, `[u16; 3]`, or `[u32; 3]`. Sub-block corners can
    /// separately be `[u8; 6]`, `[u16; 6]`, or `[u32; 6]` which each row storing
    /// $(u_{min}, v_{min}, w_{min}, u_{max}, v_{max}, w_{max})$ as indices into the regular
    /// grid within the parent block.
    pub fn array_regular_subblocks<I>(
        &mut self,
        data: I,
    ) -> Result<Array<array_type::RegularSubblock>, Error>
    where
        I: IntoIterator<Item = ([u32; 3], [u32; 6])>,
    {
        let mut parents = ParentIndices::new();
        let mut corners = RegularCorners::new();
        let mut writer = self.array_writer();
        writer.add_multiple(
            &[
                "parent_u",
                "parent_v",
                "parent_w",
                "corner_min_u",
                "corner_min_v",
                "corner_min_w",
                "corner_max_u",
                "corner_max_v",
                "corner_max_w",
            ],
            data.into_iter().map(|(p, c)| {
                parents.visit(p);
                corners.visit(c);
                (p[0], p[1], p[2], c[0], c[1], c[2], c[3], c[4], c[5])
            }),
        )?;
        Ok(self
            .array_write(writer)?
            .add_write_checks(parents.get())
            .add_write_checks(corners.get()))
    }

    /// Write an [`array_type::FreeformSubblock`](crate::array_type::FreeformSubblock) array.
    ///
    /// The `parent_indices` and `corners` iterators must be the same length. Each row is
    /// `[parent_i, parent_j, parent_k, min_corner_x, min_corner_y, min_corner_z,
    /// max_corner_x, max_corner_y, max_corner_z]`.
    ///
    /// Parent indices can be `[u8; 3]`, `[u16; 3]`, or `[u32; 3]`. Sub-block corners can be
    /// `[f32; 6]` or `[f64; 6]` which each row storing
    /// $(u_{min}, v_{min}, w_{min}, u_{max}, v_{max}, w_{max})$ in the range [0, 1] relative
    /// to the parent block.
    pub fn array_freeform_subblocks<I, C>(
        &mut self,
        data: I,
    ) -> Result<Array<array_type::FreeformSubblock>, Error>
    where
        I: IntoIterator<Item = ([u32; 3], [C; 6])>,
        C: FloatType,
    {
        let mut corner = FreeformCorners::new();
        let mut parent = ParentIndices::new();
        let mut writer = self.array_writer();
        writer.add_multiple(
            &[
                "parent_u",
                "parent_v",
                "parent_w",
                "corner_min_u",
                "corner_min_v",
                "corner_min_w",
                "corner_max_u",
                "corner_max_v",
                "corner_max_w",
            ],
            data.into_iter().map(|(p, c)| {
                parent.visit(p);
                corner.visit(c);
                (p[0], p[1], p[2], c[0], c[1], c[2], c[3], c[4], c[5])
            }),
        )?;
        Ok(self
            .array_write(writer)?
            .add_write_checks(parent.get())
            .add_write_checks(corner.get()))
    }

    /// Write an [`array_type::Number`](crate::array_type::Number) array.
    ///
    /// Values are `Option<T>` where `T` can be `f32`, `f64`, `i32`, `i64`, `chrono::NaiveDate`,
    /// or `chrono::DateTime<Utc>`. Use `None` to represent null values rather than NaN or
    /// any flag values like −9999.
    pub fn array_numbers<I, T>(&mut self, data: I) -> Result<Array<array_type::Number>, Error>
    where
        I: IntoIterator<Item = Option<T>>,
        T: NumberType,
    {
        let mut writer = self.array_writer();
        writer.add_nullable("number", data)?;
        self.array_write(writer)
    }

    /// Write an [`array_type::Index`](crate::array_type::Index) array.
    ///
    /// Values are `Option<T>` where `T` can be `Option<u8>`, `Option<u16>`, or `Option<u32>`.
    /// Smaller types won't compress much better but will let other applications allocate less
    /// memory when reading the array. Use `None` to represent null values.
    pub fn array_indices<I>(&mut self, data: I) -> Result<Array<array_type::Index>, Error>
    where
        I: IntoIterator<Item = Option<u32>>,
    {
        let mut max = MaximumIndex::new();
        let mut writer = self.array_writer();
        writer.add_nullable("index", data.into_iter().map(|v| max.visit_opt(v)))?;
        Ok(self.array_write(writer)?.add_write_checks(max.get()))
    }

    /// Write a [`array_type::Vector`](crate::array_type::Vector) array.
    ///
    /// Values are `Option<T>` where `T` can be `[f32; 2]`, `[f64; 2]`, `[f32; 3]`, or `[f64; 3]`.
    pub fn array_vectors<I, T, V>(&mut self, data: I) -> Result<Array<array_type::Vector>, Error>
    where
        I: IntoIterator<Item = Option<V>>,
        V: VectorSource<T>,
        T: FloatType,
    {
        let mut writer = self.array_writer();
        if V::IS_3D {
            writer.add_nullable_group(
                "vector",
                &["x", "y", "z"],
                data.into_iter().map(|o| o.map(V::into_3d)),
            )?;
        } else {
            writer.add_nullable_group(
                "vector",
                &["x", "y"],
                data.into_iter().map(|o| o.map(V::into_2d)),
            )?;
        }
        self.array_write(writer)
    }

    /// Write a [`array_type::Text`](crate::array_type::Text) array.
    pub fn array_text<I>(&mut self, data: I) -> Result<Array<array_type::Text>, Error>
    where
        I: IntoIterator<Item = Option<String>>,
    {
        let mut writer = self.array_writer();
        writer.add_nullable("text", data)?;
        self.array_write(writer)
    }

    /// Write a [`array_type::Boolean`](crate::array_type::Boolean) array.
    pub fn array_booleans<I>(&mut self, data: I) -> Result<Array<array_type::Boolean>, Error>
    where
        I: IntoIterator<Item = Option<bool>>,
    {
        let mut writer = self.array_writer();
        writer.add_nullable("bool", data)?;
        self.array_write(writer)
    }

    /// Write a [`array_type::Color`](crate::array_type::Color) array.
    ///
    /// Values are `Option<[u8; 4]>` with channels in RGBA order.
    pub fn array_colors<I>(&mut self, data: I) -> Result<Array<array_type::Color>, Error>
    where
        I: IntoIterator<Item = Option<[u8; 4]>>,
    {
        let mut writer = self.array_writer();
        writer.add_nullable_group("color", &["r", "g", "b", "a"], data.into_iter())?;
        self.array_write(writer)
    }
}
