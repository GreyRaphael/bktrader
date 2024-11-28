use pyo3::prelude::*;
mod base;
mod dmac;
mod grid;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let strategy = PyModule::new(parent_module.py(), "strategy")?;
    strategy.add_class::<grid::GridPercent>()?;
    strategy.add_class::<grid::GridATR>()?;
    strategy.add_class::<dmac::DMAStrategy>()?;
    parent_module.add_submodule(&strategy)
}
