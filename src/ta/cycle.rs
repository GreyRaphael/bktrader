use super::{ma::WMA, rolling::Container};
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
        // Quadrature (QQ): Uses the current Detrender value, i.e., Detrender[0]Detrender[0].
        let detrender0 = self.coeff_a * self.container.get(0) + self.coeff_b * self.container.get(2) - self.coeff_b * self.container.get(4) - self.coeff_a * self.container.get(6);
        // let detrender1 = self.coeff_a * self.container.get(1) + self.coeff_b * self.container.get(3) - self.coeff_b * self.container.get(5) - self.coeff_a * self.container.get(7);
        // let detrender2 = self.coeff_a * self.container.get(2) + self.coeff_b * self.container.get(4) - self.coeff_b * self.container.get(6) - self.coeff_a * self.container.get(8);
        // InPhase (II): Uses the Detrender value lagged by 3 periods
        let detrender3 = self.coeff_a * self.container.get(3) + self.coeff_b * self.container.get(5) - self.coeff_b * self.container.get(7) - self.coeff_a * self.container.get(9);

        (detrender3, detrender0)
    }
}
