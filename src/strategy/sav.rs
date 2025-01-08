use super::base::QuoteHandler;
use crate::broker::etf::EtfBroker;
use crate::datatype::quote::{Bar, BarM};
use crate::ta::savgol::Savgol;
use pyo3::prelude::*;

#[pyclass]
pub struct SavStg {
    #[pyo3(get)]
    pub broker: EtfBroker,
    entry_amount: f64,
    available_pos_num: usize,
}

impl QuoteHandler<BarM> for SavStg {
    fn on_quote(&mut self, quote: &BarM) {
        // self.broker.update_portfolio_value(quote); # quote is not type Bar
        todo!()
    }
}

#[pymethods]
impl SavStg {
    #[new]
    #[pyo3(signature = (init_cash=5e5, max_active_pos_len=6))]
    pub fn new(init_cash: f64, max_active_pos_len: usize) -> Self {
        let origin_amount = init_cash / max_active_pos_len as f64;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            entry_amount: origin_amount,
            available_pos_num: max_active_pos_len,
        }
    }

    pub fn on_update(&mut self, quote: &BarM) {
        self.on_quote(quote);
    }
}

#[pyclass]
pub struct SavStgD {
    #[pyo3(get)]
    pub broker: EtfBroker,
    price_savgoler: Savgol,
    vol_savgoler: Savgol,
    entry_amount: f64,
    available_pos_num: usize,
}

impl QuoteHandler<Bar> for SavStgD {
    fn on_quote(&mut self, bar: &Bar) {
        let vwap = bar.amount / bar.volume;
        let (price_deriv1, price_deriv2) = self.price_savgoler.update(vwap);
        let (vol_deriv1, vol_deriv2) = self.vol_savgoler.update(vwap);
        todo!()
    }
}

#[pymethods]
impl SavStgD {
    #[new]
    #[pyo3(signature = (init_cash=5e5, price_win=20, vol_win=5, max_active_pos_len=6))]
    pub fn new(init_cash: f64, price_win: usize, vol_win: usize, max_active_pos_len: usize) -> Self {
        let origin_amount = init_cash / max_active_pos_len as f64;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            price_savgoler: Savgol::new(price_win),
            vol_savgoler: Savgol::new(vol_win),
            entry_amount: origin_amount,
            available_pos_num: max_active_pos_len,
        }
    }

    pub fn on_update(&mut self, quote: &Bar) {
        self.on_quote(quote);
    }
}
