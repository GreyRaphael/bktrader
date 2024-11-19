use pyo3::prelude::*;
mod rolling;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let ta = PyModule::new_bound(parent_module.py(), "ta")?;
    ta.add_class::<rolling::RollingMax>()?;
    ta.add_class::<rolling::RollingMin>()?;
    ta.add_class::<rolling::RollingMean>()?;
    ta.add_class::<rolling::RollingSum>()?;
    parent_module.add_submodule(&ta)
}
