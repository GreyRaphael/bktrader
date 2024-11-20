use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PositionStatus {
    Pending,
    Opened,
    Closed,
}

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
    pub status: PositionStatus,
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
            status: PositionStatus::Pending,
            volume,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
