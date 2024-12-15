use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PositionStatus {
    Opened,
    Closed,
}

#[pyclass]
#[derive(Debug, Clone, Copy)]
pub struct Position {
    #[pyo3(get)]
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
    #[pyo3(get)]
    pub pnl: f64, // gross pnl without considering commissions
    #[pyo3(get)]
    pub fees: f64,
}

#[pymethods]
impl Position {
    #[new]
    pub fn new(id: u32, entry_dt: i32, entry_price: f64, volume: f64) -> Self {
        Self {
            id,
            entry_dt,
            exit_dt: None,
            entry_price,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            status: PositionStatus::Opened,
            volume,
            pnl: 0.0,
            fees: 0.0,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
