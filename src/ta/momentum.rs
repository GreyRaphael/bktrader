use super::{
    ma::MA,
    rolling::{RollingMax, RollingMin},
};
use pyo3::prelude::*;

// CCI - Commodity Channel Index
#[pyclass]
pub struct CCI {
    tp_meaner: MA,
    deviation_meaner: MA,
}

#[pymethods]
impl CCI {
    #[new]
    #[pyo3(signature = (ma_period=14, ma_type="sma"))]
    pub fn new(ma_period: usize, ma_type: &str) -> Self {
        Self {
            tp_meaner: MA::new(ma_period, ma_type),
            deviation_meaner: MA::new(ma_period, ma_type),
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) -> f64 {
        let tp = (high + low + close) / 3.0;
        let tp_avg = self.tp_meaner.update(tp);
        let deviation_avg = self.deviation_meaner.update((tp - tp_avg).abs());
        (tp - tp_avg) / deviation_avg
    }
}

#[pyclass]
pub struct KDJ {
    low_miner: RollingMin,
    high_maxer: RollingMax,
    smoother: MA,
}

#[pymethods]
impl KDJ {
    #[new]
    #[pyo3(signature = (minmax_period=9,ma_period=3, ma_type="sma"))]
    pub fn new(minmax_period: usize, ma_period: usize, ma_type: &str) -> Self {
        Self {
            low_miner: RollingMin::new(minmax_period),
            high_maxer: RollingMax::new(minmax_period),
            smoother: MA::new(ma_period, ma_type),
        }
    }

    // price maybe close or vwap
    pub fn update(&mut self, price: f64, high: f64, low: f64) -> (f64, f64, f64) {
        let lowest_low = self.low_miner.update(low);
        let highest_high = self.high_maxer.update(high);
        let k_line = (price - lowest_low) / (highest_high - lowest_low);
        let d_line = self.smoother.update(k_line);
        let j_line = 3.0 * k_line - 2.0 * d_line;

        (k_line, d_line, j_line)
    }
}
