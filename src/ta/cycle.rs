use super::{ma::WMA, rolling::Container};
use core::f64;
use pyo3::prelude::*;

// HtPhasor - Hilbert Transform - Phasor Components
// approximation
#[pyclass]
pub struct HtPhasor {
    container: Container,
    wmaer: WMA,
    coeff_a: f64,
    coeff_b: f64,
}

#[pymethods]
impl HtPhasor {
    #[new]
    pub fn new() -> Self {
        Self {
            container: Container::new(10),
            wmaer: WMA::new(4),
            coeff_a: 0.0962,
            coeff_b: 0.5769,
        }
    }

    pub fn update(&mut self, new_val: f64) -> (f64, f64) {
        let smooth_price = self.wmaer.update(new_val);
        self.container.update(smooth_price);
        // Quadrature (Q): Uses the current Detrender value, i.e., Detrender[0].
        let detrender0 = self.coeff_a * self.container.get(9) + self.coeff_b * self.container.get(7) - self.coeff_b * self.container.get(5) - self.coeff_a * self.container.get(3);
        // let detrender1 = self.coeff_a * self.container.get(8) + self.coeff_b * self.container.get(6) - self.coeff_b * self.container.get(4) - self.coeff_a * self.container.get(2);
        // let detrender2 = self.coeff_a * self.container.get(7) + self.coeff_b * self.container.get(5) - self.coeff_b * self.container.get(3) - self.coeff_a * self.container.get(1);
        // InPhase (I): Uses the Detrender value lagged by 3 periods
        let detrender3 = self.coeff_a * self.container.get(6) + self.coeff_b * self.container.get(4) - self.coeff_b * self.container.get(2) - self.coeff_a * self.container.get(0);

        (detrender3, detrender0)
    }
}

// HtDCPeriod - Hilbert Transform - Dominant Cycle Period
// approximation
#[pyclass]
pub struct HtDCPeriod {
    ht_phasor: HtPhasor,
    container: Container,
}

#[pymethods]
impl HtDCPeriod {
    #[new]
    pub fn new() -> Self {
        Self {
            ht_phasor: HtPhasor::new(),
            container: Container::new(2),
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let (q, i) = self.ht_phasor.update(new_val);
        let degree = (q / i).atan();
        let (head, tail) = self.container.update(degree);
        2.0 * f64::consts::PI / (tail - head)
    }
}

// HtDCPhase - Hilbert Transform - Dominant Cycle Phase
// approximation
#[pyclass]
pub struct HtDCPhase {
    ht_phasor: HtPhasor,
}

#[pymethods]
impl HtDCPhase {
    #[new]
    pub fn new() -> Self {
        Self { ht_phasor: HtPhasor::new() }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let (q, i) = self.ht_phasor.update(new_val);
        (q / i).atan().to_degrees()
    }
}

// HtSine - Hilbert Transform - SineWave
// approximation
#[pyclass]
pub struct HtSine {
    ht_phasor: HtPhasor,
}

#[pymethods]
impl HtSine {
    #[new]
    pub fn new() -> Self {
        Self { ht_phasor: HtPhasor::new() }
    }

    pub fn update(&mut self, new_val: f64) -> (f64, f64) {
        let (q, i) = self.ht_phasor.update(new_val);
        let phi = (q / i).atan();
        let sine_wave = phi.sin();
        let lead_sine_wave = (phi + f64::consts::PI / 4.0).sin();
        (sine_wave, lead_sine_wave)
    }
}

// HtTrendMode - Hilbert Transform - Trend vs Cycle Mode
// approximation
#[pyclass]
pub struct HtTrendMode {
    dc_period: HtDCPeriod,
    container: Container,
}

#[pymethods]
impl HtTrendMode {
    #[new]
    pub fn new() -> Self {
        Self {
            dc_period: HtDCPeriod::new(),
            container: Container::new(2),
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let dc_period = self.dc_period.update(new_val);
        let (head, tail) = self.container.update(dc_period);
        let trend_mode = if tail > head { 1.0 } else { 0.0 };
        trend_mode
    }
}
