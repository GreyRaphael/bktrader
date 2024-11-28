use pyo3::prelude::*;
pub mod cross;
pub mod ma;
pub mod rolling;
pub mod volatility;
pub mod momentum;
pub mod cycle;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let ta = PyModule::new(parent_module.py(), "ta")?;
    ta.add_class::<rolling::RollingSum>()?;
    ta.add_class::<rolling::RollingMax>()?;
    ta.add_class::<rolling::RollingMin>()?;
    ta.add_class::<ma::SMA>()?;
    ta.add_class::<ma::WMA>()?;
    ta.add_class::<ma::EMA>()?;
    ta.add_class::<ma::DEMA>()?;
    ta.add_class::<ma::HMA>()?;
    ta.add_class::<ma::RMA>()?;
    ta.add_class::<ma::LSMA>()?;
    ta.add_class::<ma::VWMA>()?;
    ta.add_class::<ma::ALMA>()?;
    ta.add_class::<ma::MA>()?;
    ta.add_class::<cross::Crosser>()?;
    ta.add_class::<volatility::ATR>()?;
    ta.add_class::<volatility::NATR>()?;
    ta.add_class::<momentum::CCI>()?;
    parent_module.add_submodule(&ta)
}
