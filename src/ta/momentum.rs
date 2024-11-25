use pyo3::prelude::*;

use super::ma::SMA;

// CCI - Commodity Channel Index
#[pyclass]
pub struct CCI {
    tp_meaner: SMA,
    deviation_meaner: SMA,
}

#[pymethods]
impl CCI {
    #[new]
    pub fn new(period: usize) -> Self {
        Self {
            tp_meaner: SMA::new(period),
            deviation_meaner: SMA::new(period),
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) -> f64 {
        let tp = (high + low + close) / 3.0;
        let tp_avg = self.tp_meaner.update(tp);
        let deviation_avg = self.deviation_meaner.update((tp - tp_avg).abs());
        (tp - tp_avg) / deviation_avg
    }
}
