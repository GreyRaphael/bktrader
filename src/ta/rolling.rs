use pyo3::prelude::*;

pub struct ContainerIter<'a> {
    buf: &'a [f64],
    idx: usize,
    remaining: usize,
}

impl<'a> Iterator for ContainerIter<'a> {
    type Item = &'a f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            let item = &self.buf[self.idx];
            self.idx = (self.idx + 1) % self.buf.len();
            self.remaining -= 1;
            Some(item)
        }
    }
}

pub struct Container {
    buf: Vec<f64>,
    head_idx: usize,
    tail_idx: usize,
}

impl Container {
    pub fn new(n: usize) -> Self {
        Self {
            buf: vec![f64::NAN; n],
            head_idx: 0,
            tail_idx: 0,
        }
    }

    pub fn update(&mut self, new_val: f64) -> (f64, f64) {
        self.tail_idx = self.head_idx;
        self.buf[self.tail_idx] = new_val;
        self.head_idx = (self.head_idx + 1) % self.buf.len();

        (self.buf[self.head_idx], self.buf[self.tail_idx])
    }

    pub fn get(&self, idx: usize) -> f64 {
        // idx=0 is head; idx=n-1 is tail
        self.buf[(self.head_idx + idx) % self.buf.len()]
    }

    pub fn head(&self) -> f64 {
        self.buf[self.head_idx]
    }

    pub fn tail(&self) -> f64 {
        self.buf[self.tail_idx]
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn iter(&self) -> ContainerIter<'_> {
        ContainerIter {
            buf: &self.buf,
            idx: self.head_idx,
            remaining: self.buf.len(),
        }
    }
}

#[pyclass]
pub struct RollingSum {
    pub container: Container,
    nan_count: usize,
    sum: f64,
}

#[pymethods]
impl RollingSum {
    #[new]
    pub fn new(n: usize) -> Self {
        Self {
            container: Container::new(n),
            nan_count: n,
            sum: 0.0,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let old_val = self.container.head();
        self.container.update(new_val);

        if old_val.is_finite() {
            self.sum -= old_val;
        } else {
            self.nan_count -= 1;
        }

        if new_val.is_finite() {
            self.sum += new_val;
        } else {
            self.nan_count += 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            self.sum
        }
    }
}

#[pyclass]
pub struct RollingMax {
    container: Container,
    nan_count: usize,
}

#[pymethods]
impl RollingMax {
    #[new]
    pub fn new(n: usize) -> Self {
        Self {
            container: Container::new(n),
            nan_count: n,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let old_val = self.container.head();
        self.container.update(new_val);

        if new_val.is_nan() {
            self.nan_count += 1;
        }

        if old_val.is_nan() {
            self.nan_count -= 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            self.container.iter().fold(f64::NAN, |cur_max, x| if *x <= cur_max { cur_max } else { *x })
        }
    }
}

#[pyclass]
pub struct RollingMin {
    container: Container,
    nan_count: usize,
}

#[pymethods]
impl RollingMin {
    #[new]
    pub fn new(n: usize) -> Self {
        Self {
            container: Container::new(n),
            nan_count: n,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let old_val = self.container.head();
        self.container.update(new_val);

        if new_val.is_nan() {
            self.nan_count += 1;
        }

        if old_val.is_nan() {
            self.nan_count -= 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            self.container.iter().fold(f64::NAN, |cur_min, x| if *x >= cur_min { cur_min } else { *x })
        }
    }
}

#[pyclass]
pub struct RollingQuantile {
    container: Container,
    dataset: Vec<f64>,
    nan_count: usize,
    quantile: f64,
}

#[pymethods]
impl RollingQuantile {
    #[new]
    pub fn new(n: usize, quantile: f64) -> Self {
        Self {
            container: Container::new(n),
            dataset: Vec::new(),
            nan_count: n,
            quantile,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let old_val = self.container.head();
        self.container.update(new_val);

        // Update nan_count and dataset based on new_val
        if new_val.is_finite() {
            let pos = self.dataset.binary_search_by(|v| v.partial_cmp(&new_val).unwrap()).unwrap_or_else(|e| e);
            self.dataset.insert(pos, new_val);
        } else {
            self.nan_count += 1;
        }

        // Update nan_count and dataset based on old_val
        if old_val.is_finite() {
            let pos = self.dataset.binary_search_by(|v| v.partial_cmp(&old_val).unwrap()).unwrap();
            self.dataset.remove(pos);
        } else {
            self.nan_count -= 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            let index = (self.dataset.len() - 1) as f64 * self.quantile;
            let lower_index = index.floor() as usize;
            let fraction = index - lower_index as f64;

            let lower_value = self.dataset[lower_index];
            let upper_value = if lower_index + 1 < self.dataset.len() { self.dataset[lower_index + 1] } else { lower_value };

            lower_value + fraction * (upper_value - lower_value)
        }
    }
}
#[pyclass]
pub struct RollingRank {
    container: Container,
    nan_count: usize,
}

#[pymethods]
impl RollingRank {
    #[new]
    pub fn new(n: usize) -> Self {
        Self {
            container: Container::new(n),
            nan_count: n,
        }
    }

    pub fn update(&mut self, new_val: f64) -> f64 {
        let old_val = self.container.head();
        self.container.update(new_val);

        if new_val.is_nan() {
            self.nan_count += 1;
        }

        if old_val.is_nan() {
            self.nan_count -= 1;
        }

        if self.nan_count > 0 {
            f64::NAN
        } else {
            let last_element = self.container.tail();
            let count = self.container.iter().filter(|&&x| x < last_element).count();
            count as f64 / self.container.len() as f64
        }
    }
}
