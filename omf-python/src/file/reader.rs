use crate::array::{
    PyBooleanArray, PyBoundaryArray, PyColorArray, PyFreeformSubblockArray, PyGradientArray,
    PyImageArray, PyIndexArray, PyNameArray, PyNumberArray, PyRegularSubblockArray, PyScalarArray,
    PySegmentArray, PyTexcoordArray, PyTextArray, PyTriangleArray, PyVectorArray, PyVertexArray,
};
use crate::errors::OmfException;
use crate::validate::PyProblem;
use crate::PyProject;
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools as _;
use numpy::datetime::{units, Datetime};
use numpy::ndarray::Array;
use numpy::{Element, IntoPyArray as _, PyArray, PyArray1, PyArray2};
use omf::data::{Boundaries, Boundary, NumberType, Numbers, Scalars, Texcoords, Vectors, Vertices};
use omf::date_time;
use omf::error::Error::{self, IoError};
use omf::file::{Limits, Reader};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_stub_gen::derive::*;
use std::fs::File;

#[gen_stub_pyclass]
#[pyclass(name = "Limits")]
/// Memory limits for reading OMF files.
#[derive(Clone, Copy)]
pub struct PyLimits {
    #[pyo3(get, set)]
    pub json_bytes: Option<u64>,
    #[pyo3(get, set)]
    pub image_bytes: Option<u64>,
    #[pyo3(get, set)]
    pub image_dim: Option<u32>,
    #[pyo3(get, set)]
    pub validation: Option<u32>,
}

impl From<Limits> for PyLimits {
    fn from(limits: Limits) -> Self {
        Self {
            json_bytes: limits.json_bytes,
            image_bytes: limits.image_bytes,
            image_dim: limits.image_dim,
            validation: limits.validation,
        }
    }
}

impl From<PyLimits> for Limits {
    fn from(py_limits: PyLimits) -> Self {
        Self {
            json_bytes: py_limits.json_bytes,
            image_bytes: py_limits.image_bytes,
            image_dim: py_limits.image_dim,
            validation: py_limits.validation,
        }
    }
}

impl Default for PyLimits {
    fn default() -> Self {
        Self::new()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyLimits {
    #[new]
    pub fn new() -> Self {
        Limits::default().into()
    }
}

#[gen_stub_pyclass_enum]
#[pyclass(name = "BoundaryType", eq)]
#[derive(PartialEq, Eq)]
pub enum PyBoundaryType {
    Less,
    LessEqual,
}

trait IntoNullablePyElement<E: Element> {
    fn into_nullable_pyelement(self) -> E;
}

impl<T: Default + Element> IntoNullablePyElement<T> for Option<T> {
    fn into_nullable_pyelement(self) -> T {
        self.unwrap_or_default()
    }
}

impl IntoNullablePyElement<Datetime<units::Days>> for Option<NaiveDate> {
    fn into_nullable_pyelement(self) -> Datetime<units::Days> {
        self.map_or_else(Default::default, date_time::date_to_i64)
            .into()
    }
}

impl IntoNullablePyElement<Datetime<units::Microseconds>> for Option<DateTime<Utc>> {
    fn into_nullable_pyelement(self) -> Datetime<units::Microseconds> {
        self.map_or_else(Default::default, date_time::date_time_to_i64)
            .into()
    }
}

type BoundPyArray1<'py, T> = Bound<'py, PyArray1<T>>;
type BoundPyArray2<'py, T> = Bound<'py, PyArray2<T>>;

fn pyarray2_from_vec<T: Element, const N: usize>(
    py: Python<'_>,
    array: Vec<[T; N]>,
) -> PyResult<BoundPyArray2<'_, T>> {
    Ok(PyArray::from_owned_array_bound(
        py,
        Array::from_shape_vec((array.len(), N), array.into_flattened())
            .map_err(|e| PyRuntimeError::new_err(format!("failed to create shaped array ({e})")))?,
    ))
}

fn pyarray1_from_iter<T: Element, Iter: Iterator<Item = Result<T, Error>>>(
    py: Python<'_>,
    iter: Iter,
) -> PyResult<BoundPyArray1<'_, T>> {
    Ok(iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(OmfException::py_err)?
        .into_pyarray_bound(py))
}

fn pyarray2_from_iter<T: Element, const N: usize, Iter: Iterator<Item = Result<[T; N], Error>>>(
    py: Python<'_>,
    iter: Iter,
) -> PyResult<BoundPyArray2<'_, T>> {
    pyarray2_from_vec(
        py,
        iter.collect::<Result<Vec<_>, _>>()
            .map_err(OmfException::py_err)?,
    )
}

fn nullable_pyarray1_from_iter<E: Element, T, Iter: Iterator<Item = Result<Option<T>, Error>>>(
    py: Python<'_>,
    iter: Iter,
) -> PyResult<(BoundPyArray1<'_, E>, BoundPyArray1<'_, bool>)>
where
    Option<T>: IntoNullablePyElement<E>,
{
    let (mask, array): (Vec<_>, Vec<_>) = iter
        .map_ok(|e| (e.is_none(), e.into_nullable_pyelement()))
        .collect::<Result<_, _>>()
        .map_err(OmfException::py_err)?;

    Ok((array.into_pyarray_bound(py), mask.into_pyarray_bound(py)))
}

fn nullable_pyarray2_from_iter<
    T: Element + Copy,
    const N: usize,
    Iter: Iterator<Item = Result<Option<[T; N]>, Error>>,
>(
    py: Python<'_>,
    iter: Iter,
) -> PyResult<(BoundPyArray2<'_, T>, BoundPyArray1<'_, bool>)>
where
    [T; N]: Default,
{
    let (mask, array): (Vec<_>, Vec<_>) = iter
        .map_ok(|v| (v.is_none(), v.unwrap_or_default()))
        .collect::<Result<_, _>>()
        .map_err(OmfException::py_err)?;

    Ok((pyarray2_from_vec(py, array)?, mask.into_pyarray_bound(py)))
}

fn zipped_pyarray2_from_iter<
    T: Element,
    U: Element,
    const M: usize,
    const N: usize,
    Iter: Iterator<Item = Result<([T; M], [U; N]), Error>>,
>(
    py: Python<'_>,
    iter: Iter,
) -> PyResult<(BoundPyArray2<'_, T>, BoundPyArray2<'_, U>)> {
    let (first, second): (Vec<_>, Vec<_>) = iter
        .process_results(|r| r.unzip())
        .map_err(OmfException::py_err)?;

    Ok((
        pyarray2_from_vec(py, first)?,
        pyarray2_from_vec(py, second)?,
    ))
}

#[gen_stub_pyclass]
#[pyclass(name = "Reader")]
/// OMF reader object.
///
/// Typical usage pattern is:
///
/// - Create the reader object.
/// - Optional: retrieve the file version with `reader.version()`.
/// - Optional: adjust the limits with `reader.set_limits(...)`.
/// - Read the project from the file with `reader.project()`.
/// - Iterate through the project's contents to find the elements and attributes you want to load.
/// - For each of those items load the array or image data.
///
/// **Warning:**
///     When loading arrays and images from OMF files, beware of "zip bombs"
///     where data is maliciously crafted to expand to an excessive size when decompressed,
///     leading to a potential denial of service attack.
///     Use the limits provided and check arrays sizes before allocating memory.
pub struct PyReader(Reader<File>);

#[gen_stub_pymethods]
#[pymethods]
impl PyReader {
    #[new]
    /// Creates a reader from an OMF file path.
    ///
    /// Makes only the minimum number of reads to check the file header and footer.
    /// Fails with an error if an IO error occurs or the file isn’t in OMF 2 format.
    pub fn new(filepath: &str) -> PyResult<Self> {
        let file = File::open(filepath).map_err(|e| OmfException::py_err(IoError(e)))?;
        let reader = Reader::new(file).map_err(OmfException::py_err)?;
        Ok(Self(reader))
    }

    /// Returns the current limits.
    fn limits(&self) -> PyLimits {
        self.0.limits().into()
    }

    /// Sets the memory limits.
    ///
    /// These limits prevent the reader from consuming excessive system resources, which might
    /// allow denial of service attacks with maliciously crafted files. Running without limits
    /// is not recommended.
    fn set_limits(&mut self, limits: &PyLimits) {
        self.0.set_limits((*limits).into());
    }

    /// Return the version number of the file, which can only be [2, 0] right now.
    pub fn version(&self) -> [u32; 2] {
        self.0.version()
    }

    /// Reads, validates, and returns the root `Project` object from the file.
    ///
    /// Fails with an error if an IO error occurs, the `json_bytes` limit is exceeded, or validation
    /// fails. Validation warnings are returned alongside the project if successful or included
    /// with the errors if not.
    fn project(&self) -> PyResult<(PyProject, Vec<PyProblem>)> {
        let (project, problems) = self.0.project().map_err(OmfException::py_err)?;

        let problems_array: Vec<PyProblem> =
            problems.iter().map(|e| PyProblem(e.clone())).collect();
        Ok((PyProject(project), problems_array))
    }

    /// Read a Scalar array.
    pub fn array_scalars<'py>(
        &self,
        py: Python<'py>,
        array: &PyScalarArray,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self
            .0
            .array_scalars(&array.0)
            .map_err(OmfException::py_err)?
        {
            Scalars::F32(scalars) => pyarray1_from_iter(py, scalars).map(Bound::into_any),
            Scalars::F64(scalars) => pyarray1_from_iter(py, scalars).map(Bound::into_any),
        }
    }

    /// Read a Vertex array.
    pub fn array_vertices<'py>(
        &self,
        py: Python<'py>,
        array: &PyVertexArray,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self
            .0
            .array_vertices(&array.0)
            .map_err(OmfException::py_err)?
        {
            Vertices::F32(vertices) => pyarray2_from_iter(py, vertices).map(Bound::into_any),
            Vertices::F64(vertices) => pyarray2_from_iter(py, vertices).map(Bound::into_any),
        }
    }

    /// Read a Segment array.
    pub fn array_segments<'py>(
        &self,
        py: Python<'py>,
        array: &PySegmentArray,
    ) -> PyResult<BoundPyArray2<'py, u32>> {
        pyarray2_from_iter(
            py,
            self.0
                .array_segments(&array.0)
                .map_err(OmfException::py_err)?,
        )
    }

    /// Read an Index array.
    pub fn array_indices<'py>(
        &self,
        py: Python<'py>,
        array: &PyIndexArray,
    ) -> PyResult<(BoundPyArray1<'py, u32>, Bound<'py, PyAny>)> {
        nullable_pyarray1_from_iter(
            py,
            self.0
                .array_indices(&array.0)
                .map_err(OmfException::py_err)?,
        )
        .map(|(a, b)| (a, b.into_any()))
    }

    /// Read a Triangle array.
    pub fn array_triangles<'py>(
        &self,
        py: Python<'py>,
        array: &PyTriangleArray,
    ) -> PyResult<BoundPyArray2<'py, u32>> {
        pyarray2_from_iter(
            py,
            self.0
                .array_triangles(&array.0)
                .map_err(OmfException::py_err)?,
        )
    }

    /// Read a Color array.
    pub fn array_color<'py>(
        &self,
        py: Python<'py>,
        array: &PyColorArray,
    ) -> PyResult<(BoundPyArray2<'py, u8>, Bound<'py, PyAny>)> {
        nullable_pyarray2_from_iter(
            py,
            self.0
                .array_colors(&array.0)
                .map_err(OmfException::py_err)?,
        )
        .map(|(a, b)| (a, b.into_any()))
    }

    /// Read a Gradient array.
    pub fn array_gradient<'py>(
        &self,
        py: Python<'py>,
        array: &PyGradientArray,
    ) -> PyResult<BoundPyArray2<'py, u8>> {
        pyarray2_from_iter(
            py,
            self.0
                .array_gradient(&array.0)
                .map_err(OmfException::py_err)?,
        )
    }

    /// Read a Name array.
    pub fn array_names(&self, array: &PyNameArray) -> PyResult<Vec<String>> {
        let names = self.0.array_names(&array.0).map_err(OmfException::py_err)?;
        names
            .collect::<Result<Vec<_>, _>>()
            .map_err(OmfException::py_err)
    }

    /// Read a Texcoord array.
    pub fn array_texcoords<'py>(
        &self,
        py: Python<'py>,
        array: &PyTexcoordArray,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self
            .0
            .array_texcoords(&array.0)
            .map_err(OmfException::py_err)?
        {
            Texcoords::F32(texcoords) => pyarray2_from_iter(py, texcoords).map(Bound::into_any),
            Texcoords::F64(texcoords) => pyarray2_from_iter(py, texcoords).map(Bound::into_any),
        }
    }

    /// Read bytes of an Image.
    pub fn image_bytes<'p>(
        &self,
        py: Python<'p>,
        array: &PyImageArray,
    ) -> PyResult<Bound<'p, PyBytes>> {
        self.0
            .array_bytes(&array.0)
            .map_err(OmfException::py_err)
            .map(|b| PyBytes::new_bound(py, &b))
    }

    /// Read a Number array.
    pub fn array_numbers<'py>(
        &self,
        py: Python<'py>,
        array: &PyNumberArray,
    ) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
        match self
            .0
            .array_numbers(&array.0)
            .map_err(OmfException::py_err)?
        {
            Numbers::F32(numbers) => {
                nullable_pyarray1_from_iter(py, numbers).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Numbers::F64(numbers) => {
                nullable_pyarray1_from_iter(py, numbers).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Numbers::I64(numbers) => {
                nullable_pyarray1_from_iter(py, numbers).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Numbers::Date(numbers) => {
                nullable_pyarray1_from_iter(py, numbers).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Numbers::DateTime(numbers) => {
                nullable_pyarray1_from_iter(py, numbers).map(|(a, b)| (a.into_any(), b.into_any()))
            }
        }
    }

    /// Read a Vector array.
    pub fn array_vectors<'py>(
        &self,
        py: Python<'py>,
        array: &PyVectorArray,
    ) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
        match self
            .0
            .array_vectors(&array.0)
            .map_err(OmfException::py_err)?
        {
            Vectors::F32x2(vectors) => {
                nullable_pyarray2_from_iter(py, vectors).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Vectors::F64x2(vectors) => {
                nullable_pyarray2_from_iter(py, vectors).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Vectors::F32x3(vectors) => {
                nullable_pyarray2_from_iter(py, vectors).map(|(a, b)| (a.into_any(), b.into_any()))
            }
            Vectors::F64x3(vectors) => {
                nullable_pyarray2_from_iter(py, vectors).map(|(a, b)| (a.into_any(), b.into_any()))
            }
        }
    }

    /// Read a Text array.
    pub fn array_text(&self, array: &PyTextArray) -> PyResult<Vec<Option<String>>> {
        self.0
            .array_text(&array.0)
            .map_err(OmfException::py_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(OmfException::py_err)
    }

    /// Read a Boolean array.
    pub fn array_booleans<'py>(
        &self,
        py: Python<'py>,
        array: &PyBooleanArray,
    ) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
        nullable_pyarray1_from_iter(
            py,
            self.0
                .array_booleans(&array.0)
                .map_err(OmfException::py_err)?,
        )
        .map(|(a, b)| (a.into_any(), b.into_any()))
    }

    /// Read a Boundary array.
    pub fn array_boundaries(
        &self,
        py: Python<'_>,
        array: &PyBoundaryArray,
    ) -> PyResult<Vec<(Py<PyAny>, PyBoundaryType)>> {
        fn map_boundary<T: IntoPy<Py<PyAny>> + NumberType>(
            py: Python<'_>,
            b: Boundary<T>,
        ) -> (Py<PyAny>, PyBoundaryType) {
            match b {
                Boundary::Less(v) => (v.into_py(py), PyBoundaryType::Less),
                Boundary::LessEqual(v) => (v.into_py(py), PyBoundaryType::LessEqual),
            }
        }

        let boundaries: Result<Vec<_>, _> = match self
            .0
            .array_boundaries(&array.0)
            .map_err(OmfException::py_err)?
        {
            Boundaries::F32(boundaries) => boundaries.map_ok(|b| map_boundary(py, b)).collect(),
            Boundaries::F64(boundaries) => boundaries.map_ok(|b| map_boundary(py, b)).collect(),
            Boundaries::I64(boundaries) => boundaries.map_ok(|b| map_boundary(py, b)).collect(),
            Boundaries::Date(boundaries) => boundaries.map_ok(|b| map_boundary(py, b)).collect(),
            Boundaries::DateTime(boundaries) => {
                boundaries.map_ok(|b| map_boundary(py, b)).collect()
            }
        };
        boundaries.map_err(OmfException::py_err)
    }

    /// Read a RegularSubblock array.
    pub fn array_regular_subblocks<'py>(
        &self,
        py: Python<'py>,
        array: &PyRegularSubblockArray,
    ) -> PyResult<(BoundPyArray2<'py, u32>, BoundPyArray2<'py, u32>)> {
        zipped_pyarray2_from_iter(
            py,
            self.0
                .array_regular_subblocks(&array.0)
                .map_err(OmfException::py_err)?,
        )
    }

    /// Read a FreeformSubblock array.
    pub fn array_freeform_subblocks<'py>(
        &self,
        py: Python<'py>,
        array: &PyFreeformSubblockArray,
    ) -> PyResult<(BoundPyArray2<'py, u32>, Bound<'py, PyAny>)> {
        match self
            .0
            .array_freeform_subblocks(&array.0)
            .map_err(OmfException::py_err)?
        {
            omf::data::FreeformSubblocks::F32(subblocks) => {
                zipped_pyarray2_from_iter(py, subblocks).map(|(a, b)| (a, b.into_any()))
            }
            omf::data::FreeformSubblocks::F64(subblocks) => {
                zipped_pyarray2_from_iter(py, subblocks).map(|(a, b)| (a, b.into_any()))
            }
        }
    }
}
