use super::base::QuoteHandler;
use crate::broker::etf::EtfBroker;
use crate::datatype::quote::Bar;
use crate::ta::ma::MA;
use pyo3::prelude::*;

// Dual Moving Average Crossover Strategy

#[pyclass]
pub struct DMAStrategy {
    #[pyo3(get)]
    broker: EtfBroker,
    fast_ma: MA,
    slow_ma: MA,
    entry_size: f64,
}

impl QuoteHandler<Bar> for DMAStrategy {
    fn on_quote(&mut self, bar: &Bar) {
        let vwap = bar.amount / bar.volume;
        let sma5 = self.fast_ma.update(vwap);
        let sma20 = self.slow_ma.update(vwap);

        if self.broker.active_position_len() < 5 {
            // buy
            if sma5 > sma20 {
                self.broker.entry(bar, vwap, self.entry_size, None, None);
            }
        } else if self.broker.active_position_len() > 0 {
            // sell
            if sma5 < sma20 {
                if let Some(pos) = self.broker.active_position_first() {
                    self.broker.exit(bar, vec![pos.id], vwap);
                }
            }
        }
    }
}

#[pymethods]
impl DMAStrategy {
    #[new]
    #[pyo3(signature = (init_cash=5e5, fast_period=5, slow_period=20, ma_type="sma", max_active_pos_len=6))]
    pub fn new(init_cash: f64, fast_period: usize, slow_period: usize, ma_type: &str, max_active_pos_len: usize) -> Self {
        let original_size = (init_cash / max_active_pos_len as f64 / 100.0).floor() * 100.0;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            fast_ma: MA::new(fast_period, ma_type),
            slow_ma: MA::new(slow_period, ma_type),
            entry_size: original_size,
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        self.on_quote(bar);
        self.broker.update_portfolio_value(bar);
        println!("portfolio={} at {:?}", self.broker.portfolio_value, bar);
    }
}
