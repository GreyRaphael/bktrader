use pyo3::prelude::*;
mod datatype;
mod broker;
mod ta;

#[pymodule]
fn bktrader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    datatype::register(m)?;
    broker::register(m)?;
    ta::register(m)?;
    Ok(())
}