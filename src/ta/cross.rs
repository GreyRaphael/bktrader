use super::rolling::Container;
use pyo3::prelude::*;

#[pyclass]
pub struct Crosser {
    pub x_container: Container,
    pub y_container: Container,
}

#[pymethods]
impl Crosser {
    #[new]
    pub fn new() -> Self {
        Self {
            x_container: Container::new(2),
            y_container: Container::new(2),
        }
    }

    pub fn update(&mut self, x: f64, y: f64) -> i8 {
        let (x_head, x_tail) = self.x_container.update(x);
        let (y_head, y_tail) = self.y_container.update(y);
        if (x_head > y_head) && (x_tail < y_tail) {
            // x cross down y
            -1
        } else if (x_head < y_head) && (x_tail > y_tail) {
            // x cross up y
            1
        } else {
            0
        }
    }
}
