use super::rolling::Container;
use nalgebra::{DMatrix, DVector};
use pyo3::prelude::*;

// Savgol - Savgol filter
#[pyclass]
pub struct Savgol {
    container: Container,
    derivative1_coeffs: Vec<f64>,
    derivative2_coeffs: Vec<f64>,
}

#[pymethods]
impl Savgol {
    #[new]
    pub fn new(period: usize) -> Self {
        Self {
            container: Container::new(period),
            derivative1_coeffs: savgol_coeffs(period, 2, 1, 1.0),
            derivative2_coeffs: savgol_coeffs(period, 2, 2, 1.0),
        }
    }

    pub fn update(&mut self, new_val: f64) -> (f64, f64) {
        self.container.update(new_val);
        let deriv1 = self.derivative1_coeffs.iter().zip(self.container.iter()).map(|(a, b)| a * b).sum();
        let deriv2 = self.derivative2_coeffs.iter().zip(self.container.iter()).map(|(a, b)| a * b).sum();
        (deriv1, deriv2)
    }
}

/// Compute the factorial of a number.
fn factorial(n: usize) -> usize {
    (1..=n).product()
}

/// Compute the Savitzky-Golay filter coefficients at right position.
/// visit https://docs.scipy.org/doc/scipy-1.15.0/reference/generated/scipy.signal.savgol_coeffs.html source code for more information.
///
/// # Arguments
/// * `window_length` - The length of the filter window.
/// * `polyorder` - The order of the polynomial to fit.
/// * `deriv` - The derivative to compute (default is 0).
/// * `delta` - The spacing between samples (default is 1.0).
///
/// # Returns
/// A vector of filter coefficients
fn savgol_coeffs(window_length: usize, polyorder: usize, deriv: usize, delta: f64) -> Vec<f64> {
    // polyorder must be less than window_length.
    assert_eq!(polyorder < window_length, true);
    if deriv > polyorder {
        return vec![0.0; window_length];
    }

    // Create the special vector(x) at right position
    let x: Vec<f64> = (1 - window_length as isize..=0).map(|i| i as f64).collect();
    // Form the design matrix (A)
    let mut a = DMatrix::zeros(polyorder + 1, window_length);

    // element power based on polyorder
    for row in 0..=polyorder {
        for (col, xi) in x.iter().enumerate() {
            a[(row, col)] = xi.powi(row as i32);
        }
    }

    // Form the target vector (y)
    let mut y = DVector::zeros(polyorder + 1);
    y[deriv] = factorial(deriv) as f64 / delta.powi(deriv as i32);
    // Solve the least-squares problem (A * x = y)
    let coeffs = a.svd(true, true).solve(&y, 1e-8).expect("solve matrix error");

    coeffs.as_slice().to_vec()
}
