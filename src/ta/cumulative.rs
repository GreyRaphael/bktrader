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

#[pyclass]
pub struct CumMedian {
    values: Vec<f64>,
}

#[pymethods]
impl CumMedian {
    #[new]
    pub fn new() -> Self {
        Self { values: Vec::with_capacity(512) }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        if new_val.is_finite() {
            let pos = self.values.binary_search_by(|v| v.partial_cmp(&new_val).unwrap()).unwrap_or_else(|e| e);
            self.values.insert(pos, new_val);
        }
        self.median()
    }

    fn median(&self) -> f64 {
        let length = self.values.len();
        if length == 0 {
            return f64::NAN;
        }

        if length % 2 == 0 {
            let mid = length / 2;
            (self.values[mid - 1] + self.values[mid]) / 2.0
        } else {
            self.values[length / 2]
        }
    }
}

#[pyclass]
pub struct CumQuantile {
    quantile: f64,
    dataset: Vec<f64>,
}

#[pymethods]
impl CumQuantile {
    #[new]
    pub fn new(quantile: f64) -> Self {
        Self {
            quantile,
            dataset: Vec::with_capacity(512),
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        if new_val.is_finite() {
            let pos = self.dataset.binary_search_by(|v| v.partial_cmp(&new_val).unwrap()).unwrap_or_else(|e| e);
            self.dataset.insert(pos, new_val);
        }
        self.quantile()
    }

    fn quantile(&self) -> f64 {
        let length = self.dataset.len();
        if length == 0 {
            return f64::NAN;
        }

        let index = (self.dataset.len() - 1) as f64 * self.quantile;
        let lower_index = index.floor() as usize;
        let fraction = index - lower_index as f64;

        let lower_value = self.dataset[lower_index];
        let upper_value = if lower_index + 1 < self.dataset.len() { self.dataset[lower_index + 1] } else { lower_value };

        lower_value + fraction * (upper_value - lower_value)
    }
}
