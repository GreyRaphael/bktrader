use super::ma::MA;
use pyo3::prelude::*;

// ATR - Average True Range
#[pyclass]
pub struct ATR {
    smooth_ma: MA,
}

#[pymethods]
impl ATR {
    #[new]
    #[pyo3(signature = (ma_period=21, ma_type="rma"))]
    pub fn new(ma_period: usize, ma_type: &str) -> Self {
        Self {
            smooth_ma: MA::new(ma_period, ma_type),
        }
    }

    pub fn update(&mut self, high: f64, low: f64, preclose: f64) -> f64 {
        let tr = (high - low).max((high - preclose).abs()).max((low - preclose).abs());
        self.smooth_ma.update(tr)
    }
}
