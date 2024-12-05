use pyo3::prelude::*;
pub mod position;
pub mod quote;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let datatype = PyModule::new(parent_module.py(), "datatype")?;
    datatype.add_class::<quote::Bar>()?;
    datatype.add_class::<position::Position>()?;
    parent_module.add_submodule(&datatype)
}
