use super::base::QuoteHandler;
use crate::broker::etf::EtfBroker;
use crate::datatype::quote::{Bar, BarM};
use crate::ta::rolling::Container;
use crate::ta::savgol::Savgol;
use pyo3::prelude::*;

#[pyclass]
#[allow(dead_code)]
pub struct SavStg {
    #[pyo3(get)]
    pub broker: EtfBroker,
    entry_amount: f64,
    available_pos_num: usize,
}

impl QuoteHandler<BarM> for SavStg {
    fn on_quote(&mut self, _quote: &BarM) {
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
    pd1_differ: Container,
    vd1_differ: Container,
    entry_amount: f64,
    available_pos_num: usize,
}

impl QuoteHandler<Bar> for SavStgD {
    fn on_quote(&mut self, bar: &Bar) {
        let vwap = bar.amount / bar.volume;
        let (pd1, pd2) = self.price_savgoler.update(vwap);
        let (vd1, _vd2) = self.vol_savgoler.update(bar.volume);
        let (pd1_head, pd1_tail) = self.pd1_differ.update(pd1);
        let (_vd1_head, _vd1_tail) = self.vd1_differ.update(vd1);

        if (pd1_head > 0.0) && (pd1_tail <= 0.0) {
            let positions_to_exit: Vec<u32> = self.broker.active_positions().iter().map(|pos| pos.id).collect();
            if !positions_to_exit.is_empty() {
                self.available_pos_num += positions_to_exit.len();
                self.broker.exit(bar, positions_to_exit, vwap);
            }
        }

        if self.available_pos_num > 0 {
            if (pd1_head <= 0.0) && (pd1_tail > 0.0) && (pd2 > 0.0) {
                let entry_size = (self.entry_amount / vwap / 100.0).floor() * 100.0;
                self.broker.entry(bar, vwap, entry_size, None, None);
                self.available_pos_num -= 1;
            }
        }

        self.broker.update_portfolio_value(bar);
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
            pd1_differ: Container::new(2),
            vd1_differ: Container::new(2),
            entry_amount: origin_amount,
            available_pos_num: max_active_pos_len,
        }
    }

    pub fn on_update(&mut self, quote: &Bar) {
        self.on_quote(quote);
    }
}
