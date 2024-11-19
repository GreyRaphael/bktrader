use pyo3::prelude::*;
mod datatype;
mod broker;

#[pymodule]
fn bktrader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    datatype::register(m)?;
    broker::register(m)?;
    Ok(())
}