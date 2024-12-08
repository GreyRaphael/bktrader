use pyo3::prelude::*;

#[pyclass]
pub struct CumMax {
    max: f64,
}

#[pymethods]
impl CumMax {
    #[new]
    pub fn new() -> Self {
        Self { max: f64::NEG_INFINITY }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        self.max = f64::max(new_val, self.max);
        self.max
    }
}

#[pyclass]
pub struct CumMin {
    min: f64,
}

#[pymethods]
impl CumMin {
    #[new]
    pub fn new() -> Self {
        Self { min: f64::INFINITY }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        self.min = f64::min(new_val, self.min);
        self.min
    }
}

#[pyclass]
pub struct CumMinMax {
    min: f64,
    max: f64,
}

#[pymethods]
impl CumMinMax {
    #[new]
    pub fn new(init_min: f64, init_max: f64) -> Self {
        Self {
            min: init_min, // f64::INFINITY
            max: init_max, // f64::NEG_INFINITY,
        }
    }

    pub fn update(&mut self, new_val: f64) -> (f64, f64) {
        self.min = f64::min(new_val, self.min);
        self.max = f64::max(new_val, self.max);
        (self.min, self.max)
    }
}

#[pyclass]
pub struct CumMean {
    count: usize,
    sum: f64,
}

#[pymethods]
impl CumMean {
    #[new]
    pub fn new() -> Self {
        Self { count: 0, sum: 0.0 }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        if new_val.is_finite() {
            self.count += 1;
            self.sum += new_val;
        }

        self.sum / self.count as f64
    }
}
