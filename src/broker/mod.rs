use pyo3::prelude::*;
pub mod etf;

pub fn register(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let broker = PyModule::new_bound(parent_module.py(), "broker")?;
    broker.add_class::<etf::EtfBroker>()?;
    parent_module.add_submodule(&broker)
}
