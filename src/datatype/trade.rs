use pyo3::prelude::*;

// History Trade
// volume>0, means buy
// volume<0, means sell
#[pyclass]
#[derive(Debug, Clone)]
pub struct Trade {
    #[pyo3(get)]
    pub code: u32,
    #[pyo3(get)]
    pub dt: i32,
    #[pyo3(get)]
    pub price: f64,
    #[pyo3(get)]
    pub volume: f64,
}

#[pymethods]
impl Trade {
    #[new]
    fn new(code: u32, dt: i32, price: f64, volume: f64) -> Self {
        Trade {
            code,
            dt,
            price,
            volume,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
