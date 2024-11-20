use pyo3::prelude::*;

// Opened or Closed Position
#[pyclass]
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub id: u32,
    #[pyo3(get)]
    pub entry_dt: i32,
    #[pyo3(get)]
    pub exit_dt: Option<i32>,
    #[pyo3(get)]
    pub entry_price: f64,
    #[pyo3(get)]
    pub exit_price: Option<f64>,
    #[pyo3(get)]
    pub stop_loss: Option<f64>,
    #[pyo3(get)]
    pub take_profit: Option<f64>,
    #[pyo3(get)]
    pub status: u8, // 0 pending, 1 oepn, 2 close
    #[pyo3(get)]
    pub volume: f64,
}

#[pymethods]
impl Position {
    #[new]
    pub fn new(entry_dt: i32, entry_price: f64, volume: f64) -> Self {
        Position {
            id: 0,
            entry_dt,
            exit_dt: None,
            entry_price,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            status: 0,
            volume,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
