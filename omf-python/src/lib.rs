/// Python bindings.
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

mod array;
mod attribute;
mod block_model;
mod colormap;
mod element;
mod errors;
mod file;
mod geometry;
mod grid;
mod omf1;
mod project;
mod validate;

/// Returns the version of the library
#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// This module provides python bindings for omf-rust.
#[pymodule]
fn omf_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<array::PyBooleanArray>()?;
    m.add_class::<array::PyBoundaryArray>()?;
    m.add_class::<array::PyColorArray>()?;
    m.add_class::<array::PyFreeformSubblockArray>()?;
    m.add_class::<array::PyGradientArray>()?;
    m.add_class::<array::PyImageArray>()?;
    m.add_class::<array::PyIndexArray>()?;
    m.add_class::<array::PyNameArray>()?;
    m.add_class::<array::PyNumberArray>()?;
    m.add_class::<array::PyRegularSubblockArray>()?;
    m.add_class::<array::PyScalarArray>()?;
    m.add_class::<array::PySegmentArray>()?;
    m.add_class::<array::PyTexcoordArray>()?;
    m.add_class::<array::PyTextArray>()?;
    m.add_class::<array::PyTriangleArray>()?;
    m.add_class::<array::PyVectorArray>()?;
    m.add_class::<array::PyVertexArray>()?;
    m.add_class::<attribute::PyAttribute>()?;
    m.add_class::<attribute::PyAttributeDataBoolean>()?;
    m.add_class::<attribute::PyAttributeDataCategory>()?;
    m.add_class::<attribute::PyAttributeDataColor>()?;
    m.add_class::<attribute::PyAttributeDataMappedTexture>()?;
    m.add_class::<attribute::PyAttributeDataNumber>()?;
    m.add_class::<attribute::PyAttributeDataProjectedTexture>()?;
    m.add_class::<attribute::PyAttributeDataText>()?;
    m.add_class::<attribute::PyAttributeDataVector>()?;
    m.add_class::<attribute::PyLocation>()?;
    m.add_class::<block_model::PyBlockModel>()?;
    m.add_class::<block_model::PyFreeformSubblocks>()?;
    m.add_class::<block_model::PyRegularSubblocks>()?;
    m.add_class::<block_model::PySubblockMode>()?;
    m.add_class::<colormap::PyNumberColormapContinuous>()?;
    m.add_class::<colormap::PyNumberColormapDiscrete>()?;
    m.add_class::<element::PyElement>()?;
    m.add_class::<file::reader::PyBoundaryType>()?;
    m.add_class::<file::reader::PyLimits>()?;
    m.add_class::<file::reader::PyReader>()?;
    m.add_class::<geometry::PyGridSurface>()?;
    m.add_class::<geometry::PyLineSet>()?;
    m.add_class::<geometry::PyPointSet>()?;
    m.add_class::<geometry::PySurface>()?;
    m.add_class::<grid::PyGrid2Regular>()?;
    m.add_class::<grid::PyGrid2Tensor>()?;
    m.add_class::<grid::PyGrid3Regular>()?;
    m.add_class::<grid::PyGrid3Tensor>()?;
    m.add_class::<grid::PyOrient2>()?;
    m.add_class::<grid::PyOrient3>()?;
    m.add_class::<omf1::converter::PyOmf1Converter>()?;
    m.add_class::<project::PyProject>()?;
    m.add_class::<validate::PyProblem>()?;

    m.add_function(wrap_pyfunction!(omf1::converter::detect_omf1, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    m.add("OmfException", py.get_type::<errors::OmfException>())?;
    m.add(
        "OmfFileIoException",
        py.get_type::<errors::OmfFileIoException>(),
    )?;
    m.add(
        "OmfInvalidDataException",
        py.get_type::<errors::OmfInvalidDataException>(),
    )?;
    m.add(
        "OmfJsonException",
        py.get_type::<errors::OmfJsonException>(),
    )?;
    m.add(
        "OmfLimitExceededException",
        py.get_type::<errors::OmfLimitExceededException>(),
    )?;
    m.add(
        "OmfNotSupportedException",
        py.get_type::<errors::OmfNotSupportedException>(),
    )?;
    m.add(
        "OmfValidationFailedException",
        py.get_type::<errors::OmfValidationFailedException>(),
    )?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
