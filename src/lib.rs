use pyo3::prelude::*;
mod datatype;

#[pymodule]
fn bktrader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    datatype::register(m)?;
    Ok(())
}