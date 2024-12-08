use pyo3::prelude::*;
pub mod cross;
pub mod cumulative;
pub mod cycle;
pub mod ma;
pub mod momentum;
pub mod rolling;
pub mod volatility;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let ta = PyModule::new(parent_module.py(), "ta")?;
    ta.add_class::<rolling::RollingSum>()?;
    ta.add_class::<rolling::RollingMax>()?;
    ta.add_class::<rolling::RollingMin>()?;
    ta.add_class::<rolling::RollingQuantile>()?;
    ta.add_class::<rolling::RollingRank>()?;
    ta.add_class::<rolling::RollingMean>()?;
    ta.add_class::<cumulative::CumMin>()?;
    ta.add_class::<cumulative::CumMax>()?;
    ta.add_class::<cumulative::CumMinMax>()?;
    ta.add_class::<cumulative::CumMean>()?;
    ta.add_class::<cumulative::CumMedian>()?;
    ta.add_class::<cumulative::CumQuantile>()?;
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
    ta.add_class::<momentum::KDJ>()?;
    parent_module.add_submodule(&ta)
}
