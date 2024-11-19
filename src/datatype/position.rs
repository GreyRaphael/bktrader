use pyo3::prelude::*;

// Opened or Closed Position
#[pyclass]
#[derive(Debug, Clone)]
pub struct Position {
    #[pyo3(get)]
    pub dt: i32,
    #[pyo3(get)]
    pub price: f64,
    #[pyo3(get)]
    pub volume: f64,
}

#[pymethods]
impl Position {
    #[new]
    fn new(dt: i32, price: f64, volume: f64) -> Self {
        Position {
            dt,
            price,
            volume,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
