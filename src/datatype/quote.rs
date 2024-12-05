use pyo3::prelude::*;

#[pyclass]
#[derive(Debug)]
pub struct Bar {
    #[pyo3(get)]
    pub code: u32,
    #[pyo3(get)]
    pub dt: i32,
    #[pyo3(get)]
    pub preclose: f64,
    #[pyo3(get)]
    pub open: f64,
    #[pyo3(get)]
    pub high: f64,
    #[pyo3(get)]
    pub low: f64,
    #[pyo3(get)]
    pub close: f64,
    #[pyo3(get)]
    pub netvalue: f64,
    #[pyo3(get)]
    pub volume: f64,
    #[pyo3(get)]
    pub amount: f64,
    #[pyo3(get)]
    pub trades_count: f64,
    #[pyo3(get)]
    pub turnover: f64,
}

#[pymethods]
impl Bar {
    #[new]
    #[pyo3(signature = (code=0, dt=0, preclose=0.0, open=0.0, high=0.0, low=0.0, close=0.0, netvalue=0.0, volume=0.0, amount=0.0, trades_count=0.0, turnover=0.0))]
    fn new(
        code: u32,
        dt: i32,
        preclose: f64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        netvalue: f64,
        volume: f64,
        amount: f64,
        trades_count: f64,
        turnover: f64,
    ) -> Self {
        Bar {
            code,
            dt,
            preclose,
            open,
            high,
            low,
            close,
            netvalue,
            volume,
            amount,
            trades_count,
            turnover,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
