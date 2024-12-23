use pyo3::prelude::*;
pub mod analyzer;
pub mod etf;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let broker = PyModule::new(parent_module.py(), "broker")?;
    broker.add_class::<etf::EtfBroker>()?;
    broker.add_class::<analyzer::Analyzer>()?;
    parent_module.add_submodule(&broker)
}
