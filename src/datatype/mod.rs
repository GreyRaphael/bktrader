use pyo3::prelude::*;
pub mod bar;
pub mod position;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let datatype = PyModule::new_bound(parent_module.py(), "datatype")?;
    datatype.add_class::<bar::Bar>()?;
    datatype.add_class::<position::Position>()?;
    parent_module.add_submodule(&datatype)
}