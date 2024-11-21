use super::rolling::{Container, RollingSum};
use pyo3::prelude::*;

// SMA - Simple Moving Average
#[pyclass]
pub struct SMA {
    sumer: RollingSum,
}

#[pymethods]
impl SMA {
    #[new]
    pub fn new(period: usize) -> Self {
        Self {
            sumer: RollingSum::new(period),
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        self.sumer.update(new_val) / self.sumer.container.len() as f64
    }
}

// WMA - Weighted Moving Average
#[pyclass]
pub struct WMA {
    container: Container,
    n: usize,
    nan_count: usize,
    sum: f64,
    weighted_sum: f64,
}

#[pymethods]
impl WMA {
    #[new]
    pub fn new(period: usize) -> Self {
        Self {
            container: Container::new(period),
            n: period,
            nan_count: period,
            sum: 0.0,
            weighted_sum: 0.0,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let old_val = self.container.head();
        self.container.update(new_val);

        if old_val.is_finite() {
            self.weighted_sum -= self.sum;
            self.sum -= old_val;
        } else {
            self.nan_count -= 1;
        }

        if new_val.is_finite() {
            self.weighted_sum += new_val * self.n as f64;
            self.sum += new_val;
        } else {
            self.nan_count += 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            self.weighted_sum / (self.n * (self.n + 1)) as f64 * 2.0
        }
    }
}

// EMA - Exponential Moving Average
#[pyclass]
pub struct EMA {
    alpha: f64,
    ema: Option<f64>,
}

#[pymethods]
impl EMA {
    #[new]
    pub fn new(period: usize) -> Self {
        let alpha = 2.0 / (period as f64 + 1.0);
        Self { alpha, ema: None }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        if new_val.is_finite() {
            if let Some(prev_ema) = self.ema {
                self.ema = Some(prev_ema * (1.0 - self.alpha) + new_val * self.alpha);
            } else {
                self.ema = Some(new_val);
            }
            self.ema.unwrap()
        } else {
            f64::NAN
        }
    }
}

#[pyclass]
pub struct MA {
    inner: MAType,
}

enum MAType {
    Simple(SMA),
    Weighted(WMA),
    Exponential(EMA),
}

#[pymethods]
impl MA {
    #[new]
    pub fn new(window: usize, method: &str) -> Self {
        let inner = match method {
            "sma" => MAType::Simple(SMA::new(window)),
            "wma" => MAType::Weighted(WMA::new(window)),
            "ema" => MAType::Exponential(EMA::new(window)),
            _ => panic!("Invalid method"),
        };
        MA { inner }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        match &mut self.inner {
            MAType::Simple(sma) => sma.update(new_val),
            MAType::Weighted(wma) => wma.update(new_val),
            MAType::Exponential(ema) => ema.update(new_val),
        }
    }
}
