use pyo3::prelude::*;
pub mod base;
mod dmac;
mod grid;
pub mod qdii;
mod sav;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let strategy = PyModule::new(parent_module.py(), "strategy")?;
    strategy.add_class::<grid::GridPercent>()?;
    strategy.add_class::<grid::GridATR>()?;
    strategy.add_class::<dmac::DMAStrategy>()?;
    strategy.add_class::<qdii::GridCCI>()?;
    strategy.add_class::<sav::SavStg>()?;
    parent_module.add_submodule(&strategy)
}
