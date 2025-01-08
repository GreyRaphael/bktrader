use super::base::QuoteHandler;
use crate::broker::etf::EtfBroker;
use crate::datatype::quote::BarM;
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
