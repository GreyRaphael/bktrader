use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)] // for the #[pyo3(get)]
pub struct Analyzer {
    #[pyo3(get)]
    equity_curve: Vec<f64>,
}

#[pymethods]
impl Analyzer {
    #[new]
    pub fn new() -> Self {
        Analyzer {
            equity_curve: Vec::with_capacity(1024),
        }
    }

    pub fn update(&mut self, portfolio_value: f64) {
        self.equity_curve.push(portfolio_value);
    }

    pub fn total_return(&self) -> f64 {
        if let (Some(&first), Some(&last)) = (self.equity_curve.first(), self.equity_curve.last()) {
            (last - first) / first
        } else {
            0.0
        }
    }

    // CAGR (Compound Annual Growth Rate)
    pub fn cagr(&self) -> f64 {
        let total_return = self.total_return();
        let trading_days = 243.0; // china average trading days in a year
        let periods = self.equity_curve.len() as f64 / trading_days;
        let cagr = (1.0 + total_return).powf(1.0 / periods) - 1.0;

        cagr
    }

    pub fn max_drawdown(&self) -> f64 {
        let mut running_max = std::f64::NEG_INFINITY;
        let mut max_drawdown = 0.0;

        for &portfolio_value in self.equity_curve.iter() {
            if portfolio_value > running_max {
                running_max = portfolio_value;
            } else {
                let drawdown = (running_max - portfolio_value) / running_max;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }

        max_drawdown
    }

    pub fn max_drawup(&self) -> f64 {
        let mut running_min = std::f64::INFINITY;
        let mut max_drawup = 0.0;

        for &portfolio_value in self.equity_curve.iter() {
            if portfolio_value < running_min {
                running_min = portfolio_value;
            } else {
                let drawup = (portfolio_value - running_min) / running_min;
                if drawup > max_drawup {
                    max_drawup = drawup;
                }
            }
        }

        max_drawup
    }

    pub fn sharpe_ratio(&self, risk_free_rate: f64) -> (f64, f64, f64) {
        if self.equity_curve.len() < 2 {
            return (f64::NAN, f64::NAN, f64::NAN); // or handle the error as appropriate
        }

        // Step 1: Compute Daily Returns
        let mut daily_returns = Vec::with_capacity(self.equity_curve.len() - 1);
        for i in 1..self.equity_curve.len() {
            let daily_return = (self.equity_curve[i] / self.equity_curve[i - 1]) - 1.0;
            daily_returns.push(daily_return);
        }

        // Step 2: Calculate Average Daily Return
        let sum_returns: f64 = daily_returns.iter().sum();
        let avg_daily_return = sum_returns / daily_returns.len() as f64;

        // Annualize the average daily return
        let trading_days = 243.0; // Average trading days in a year of china
        let annual_return = (1.0 + avg_daily_return).powf(trading_days) - 1.0;

        // Step 3: Calculate Daily Return Volatility (Standard Deviation)
        let mean_return = avg_daily_return;
        let variance = daily_returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / (daily_returns.len() as f64 - 1.0);
        let daily_volatility = variance.sqrt();

        // Annualize the volatility
        let annual_volatility = daily_volatility * trading_days.sqrt();

        // Step 4: Calculate Sharpe Ratio
        let sharpe_ratio = if annual_volatility != 0.0 {
            (annual_return - risk_free_rate) / annual_volatility
        } else {
            0.0 // Avoid division by zero
        };

        (annual_return, annual_volatility, sharpe_ratio)
    }

    pub fn sortino_ratio(&self, risk_free_rate: f64, mar: f64) -> (f64, f64, f64) {
        if self.equity_curve.len() < 2 {
            return (f64::NAN, f64::NAN, f64::NAN); // or handle the error as appropriate
        }

        // Step 1: Compute Daily Returns
        let mut daily_returns = Vec::with_capacity(self.equity_curve.len() - 1);
        for i in 1..self.equity_curve.len() {
            let daily_return = (self.equity_curve[i] / self.equity_curve[i - 1]) - 1.0;
            daily_returns.push(daily_return);
        }

        // Step 2: Calculate Average Daily Return
        let sum_returns: f64 = daily_returns.iter().sum();
        let avg_daily_return = sum_returns / daily_returns.len() as f64;

        // Annualize the average daily return
        let trading_days = 243.0; // Average trading days in a year
        let annual_return = (1.0 + avg_daily_return).powf(trading_days) - 1.0;

        // Step 3: Calculate Downside Deviation
        // mar: Min Acceptable Return
        let mut negative_deviations = Vec::new();
        for &return_i in &daily_returns {
            let deviation = return_i - mar / trading_days;
            if deviation < 0.0 {
                negative_deviations.push(deviation);
            }
        }

        let downside_variance = if !negative_deviations.is_empty() {
            negative_deviations.iter().map(|&d| d.powi(2)).sum::<f64>() / negative_deviations.len() as f64
        } else {
            0.0
        };

        let daily_downside_deviation = downside_variance.sqrt();

        // Annualize the downside deviation
        let annual_downside_deviation = daily_downside_deviation * trading_days.sqrt();

        // Step 4: Calculate Sortino Ratio
        let sortino_ratio = if annual_downside_deviation != 0.0 {
            (annual_return - risk_free_rate) / annual_downside_deviation
        } else {
            0.0 // Avoid division by zero
        };

        (annual_return, annual_downside_deviation, sortino_ratio)
    }
}
