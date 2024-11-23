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
        Self { sumer: RollingSum::new(period) }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        self.sumer.update(new_val) / self.sumer.container.len() as f64
    }
}

// WMA - Weighted Moving Average
#[pyclass]
pub struct WMA {
    container: Container,
    n: f64,
    sumn: f64,
    nan_count: usize,
    sum: f64,
    weighted_sum: f64,
}

#[pymethods]
impl WMA {
    #[new]
    pub fn new(period: usize) -> Self {
        let n = period as f64;
        let sumn = n * (n + 1.0) / 2.0;
        Self {
            container: Container::new(period),
            n,
            sumn,
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
            self.weighted_sum += new_val * self.n;
            self.sum += new_val;
        } else {
            self.nan_count += 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            self.weighted_sum / self.sumn
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

// DEMA - Double Exponential Moving Average
#[pyclass]
pub struct DEMA {
    ema_lv1: EMA,
    ema_lv2: EMA,
}

#[pymethods]
impl DEMA {
    #[new]
    pub fn new(period: usize) -> Self {
        Self {
            ema_lv1: EMA::new(period),
            ema_lv2: EMA::new(period),
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let lv1 = self.ema_lv1.update(new_val);
        let lv2 = self.ema_lv2.update(lv1);

        2.0 * lv1 - lv2
    }
}

// RMA - Relative Moving Average, similar to EMA
#[pyclass]
pub struct RMA {
    period: f64,
    rma: Option<f64>,
}

#[pymethods]
impl RMA {
    #[new]
    pub fn new(period: usize) -> Self {
        Self { period: period as f64, rma: None }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        if new_val.is_finite() {
            if let Some(prev_rma) = self.rma {
                self.rma = Some((prev_rma * (self.period - 1.0) + new_val) / self.period);
            } else {
                self.rma = Some(new_val);
            }
            self.rma.unwrap()
        } else {
            f64::NAN
        }
    }
}

// HMA - Hull Moving Average
#[pyclass]
pub struct HMA {
    full_wma: WMA,
    half_wma: WMA,
    sqrt_wma: WMA,
}

#[pymethods]
impl HMA {
    #[new]
    pub fn new(period: usize) -> Self {
        let sqrt_period = (period as f64).sqrt().floor() as usize;
        Self {
            full_wma: WMA::new(period),
            half_wma: WMA::new(period / 2),
            sqrt_wma: WMA::new(sqrt_period),
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let diff_ma = 2.0 * self.half_wma.update(new_val) - self.full_wma.update(new_val);
        self.sqrt_wma.update(diff_ma)
    }
}

// LSMA - Least Squares Moving Average
#[pyclass]
pub struct LSMA {
    container: Container,
    n: f64,
    sumn: f64,
    denominator: f64,
    nan_count: usize,
    sum: f64,
    weighted_sum: f64,
}

#[pymethods]
impl LSMA {
    #[new]
    pub fn new(period: usize) -> Self {
        let n = period as f64;
        let sumn = n * (n + 1.0) / 2.0;
        let denominator = n * n * (n + 1.0) * (2.0 * n + 1.0) / 6.0 - (sumn).powi(2);
        Self {
            container: Container::new(period),
            n,
            sumn,
            denominator,
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
            self.weighted_sum += new_val * self.n;
            self.sum += new_val;
        } else {
            self.nan_count += 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            let slope = (self.n * self.weighted_sum - self.sumn * self.sum) / self.denominator;
            let intercept = (self.sum - slope * self.sumn) / self.n;
            intercept + slope * self.n
        }
    }
}

// VWMA - Volume-Weighted Moving Average
#[pyclass]
pub struct VWMA {
    weighted_sum: RollingSum,
    vol_sum: RollingSum,
}

#[pymethods]
impl VWMA {
    #[new]
    pub fn new(period: usize) -> Self {
        Self {
            weighted_sum: RollingSum::new(period),
            vol_sum: RollingSum::new(period),
        }
    }

    pub fn update(&mut self, price: f64, volume: f64) -> f64 {
        let tot_amt = self.weighted_sum.update(price * volume);
        let tot_vol = self.vol_sum.update(volume);
        tot_amt / tot_vol
    }
}

// ALMA - Arnaud Legoux Moving Average
#[pyclass]
pub struct ALMA {
    container: Container,
    weights_sum: f64,
    weights: Vec<f64>,
}

#[pymethods]
impl ALMA {
    #[new]
    // offset: [0.0, 1.0]
    // 0: center is oldest point
    // 1: center is newest point
    // sigma usually is 6
    pub fn new(period: usize, offset: f64, sigma: f64) -> Self {
        let m = offset * (period as f64 - 1.0);
        let s = period as f64 / sigma;
        let mut weights = Vec::with_capacity(period);
        for i in 0..period {
            let w = (-((i as f64 - m) / s).powi(2)).exp();
            weights.push(w);
        }

        Self {
            container: Container::new(period),
            weights_sum: weights.iter().sum(),
            weights,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        self.container.update(new_val);
        let weighted_sum: f64 = self.container.iter().zip(self.weights.iter()).map(|(p, w)| p * w).sum();
        weighted_sum / self.weights_sum
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
    DoubleExponential(DEMA),
    Hull(HMA),
    Relative(RMA),
    LeastSquares(LSMA),
}

#[pymethods]
impl MA {
    #[new]
    pub fn new(window: usize, method: &str) -> Self {
        let inner = match method {
            "sma" => MAType::Simple(SMA::new(window)),
            "wma" => MAType::Weighted(WMA::new(window)),
            "ema" => MAType::Exponential(EMA::new(window)),
            "dema" => MAType::DoubleExponential(DEMA::new(window)),
            "hma" => MAType::Hull(HMA::new(window)),
            "rma" => MAType::Relative(RMA::new(window)),
            "lsma" => MAType::LeastSquares(LSMA::new(window)),
            _ => panic!("Invalid method"),
        };
        MA { inner }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        match &mut self.inner {
            MAType::Simple(sma) => sma.update(new_val),
            MAType::Weighted(wma) => wma.update(new_val),
            MAType::Exponential(ema) => ema.update(new_val),
            MAType::DoubleExponential(dema) => dema.update(new_val),
            MAType::Hull(hma) => hma.update(new_val),
            MAType::Relative(rma) => rma.update(new_val),
            MAType::LeastSquares(lsma) => lsma.update(new_val),
        }
    }
}
