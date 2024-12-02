use super::base::QuoteHandler;
use crate::broker::etf::EtfBroker;
use crate::datatype::bar::Bar;
use crate::ta::momentum::CCI;
use crate::ta::rolling::RollingRank;
use pyo3::prelude::*;

#[pyclass]
pub struct GridCCI {
    #[pyo3(get)]
    broker: EtfBroker,
    cci: CCI,
    ranker: RollingRank,
    rank_limit: f64,
    entry_amount: f64,
    available_pos_num: usize,
}

impl QuoteHandler<Bar> for GridCCI {
    fn on_quote(&mut self, bar: &Bar) {
        let vwap = bar.amount / bar.volume;
        let cci_val = self.cci.update(bar.high, bar.low, vwap);
        let cci_rank = self.ranker.update(cci_val);

        let mut positions_to_exit = Vec::new();
        for pos in self.broker.positions.iter() {
            if let Some(take_profit) = pos.take_profit {
                let profit = vwap / pos.entry_price - 1.0;
                if profit > take_profit {
                    positions_to_exit.push(pos.id);
                }
            }
        }
        if !positions_to_exit.is_empty() {
            self.broker.exit(bar, positions_to_exit, vwap);
        }

        if self.available_pos_num > 0 {
            if cci_rank <= self.rank_limit {
                let entry_size = (self.entry_amount / vwap / 100.0).floor() * 100.0;
                self.broker.entry(bar, vwap, entry_size, None, Some(0.05));
            }
        }
    }
}

#[pymethods]
impl GridCCI {
    #[new]
    #[pyo3(signature = (init_cash=5e5, rank_period=60, cci_period=20, ma_type="sma", rank_limit=0.1, max_active_pos_len=6))]
    pub fn new(init_cash: f64, rank_period: usize, cci_period: usize, ma_type: &str, rank_limit: f64, max_active_pos_len: usize) -> Self {
        let origin_amount = init_cash / max_active_pos_len as f64;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            cci: CCI::new(cci_period, ma_type),
            ranker: RollingRank::new(rank_period),
            rank_limit,
            entry_amount: origin_amount,
            available_pos_num: max_active_pos_len,
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        self.on_quote(bar);
        self.broker.update_portfolio_value(bar);
    }
}
