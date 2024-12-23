use pyo3::prelude::*;
// mod backtest;
mod broker;
pub mod datatype;
pub mod strategy;
mod ta;

#[pymodule]
fn bktrader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    datatype::register(m)?;
    broker::register(m)?;
    ta::register(m)?;
    strategy::register(m)?;
    Ok(())
}
