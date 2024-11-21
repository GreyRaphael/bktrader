use pyo3::prelude::*;
pub mod ma;
pub mod rolling;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let ta = PyModule::new_bound(parent_module.py(), "ta")?;
    ta.add_class::<rolling::RollingSum>()?;
    ta.add_class::<rolling::RollingMax>()?;
    ta.add_class::<rolling::RollingMin>()?;
    ta.add_class::<ma::SMA>()?;
    ta.add_class::<ma::WMA>()?;
    ta.add_class::<ma::EMA>()?;
    ta.add_class::<ma::MA>()?;
    parent_module.add_submodule(&ta)
}
