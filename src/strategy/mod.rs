use pyo3::prelude::*;
mod grid;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let strategy = PyModule::new_bound(parent_module.py(), "strategy")?;
    strategy.add_class::<grid::GridPercent>()?;
    strategy.add_class::<grid::GridATR>()?;
    parent_module.add_submodule(&strategy)
}
