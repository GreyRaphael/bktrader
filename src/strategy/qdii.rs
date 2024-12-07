use super::base::QuoteHandler;
use crate::broker::etf::EtfBroker;
use crate::datatype::quote::Bar;
use crate::ta::momentum::CCI;
use crate::ta::rolling::{Container, RollingRank};
use pyo3::prelude::*;

#[pyclass]
pub struct GridCCI {
    #[pyo3(get)]
    broker: EtfBroker,
    cci: CCI,
    vol_differ: Container,
    ranker: RollingRank,
    rank_limit: f64,
    entry_amount: f64,
    max_pos_num: usize,
    available_pos_num: usize,
    profit_limit: f64,
    loss_limit: f64,
}

impl QuoteHandler<Bar> for GridCCI {
    fn on_quote(&mut self, bar: &Bar) {
        // in real-time quote, amount & volume should be a predicted value by real-time amount & volume
        let vwap = bar.amount / bar.volume;
        let cci_val = self.cci.update(bar.high, bar.low, vwap);
        let cci_rank = self.ranker.update(cci_val);
        let (vol_head, vol_tail) = self.vol_differ.update(bar.volume);

        let mut positions_to_exit = Vec::new();
        for pos in self.broker.active_positions().iter() {
            let profit = vwap / pos.entry_price - 1.0;
            if let Some(take_profit) = pos.take_profit {
                if profit > take_profit {
                    positions_to_exit.push(pos.id);
                    self.available_pos_num += 1;
                }
            }
            if let Some(stop_loss) = pos.stop_loss {
                if profit < stop_loss {
                    positions_to_exit.push(pos.id);
                    self.available_pos_num += 1;
                }
            }
        }
        if !positions_to_exit.is_empty() {
            self.broker.exit(bar, positions_to_exit, vwap);
        }

        if self.available_pos_num > 0 {
            // println!("prev_vol: {}, vol:{}, cci:{}, cci_rank:{}, dt:{}", vol_head, vol_tail, cci_val, cci_rank, bar.dt);
            if (vol_tail / vol_head < 1.0) && (cci_val < -0.5) && (cci_rank < self.rank_limit) {
                let multiplier = 1.1_f64.powi((self.max_pos_num - self.available_pos_num) as i32);
                let entry_size = (self.entry_amount * multiplier / vwap / 100.0).floor() * 100.0;
                self.broker.entry(bar, vwap, entry_size, Some(self.loss_limit), Some(self.profit_limit));
                self.available_pos_num -= 1;
            }
        }
    }
}

#[pymethods]
impl GridCCI {
    #[new]
    #[pyo3(signature = (init_cash=5e5, rank_period=60, cci_period=20, ma_type="sma", rank_limit=0.1, max_active_pos_len=6, profit_limit=0.1, loss_limit=-1.0))]
    pub fn new(init_cash: f64, rank_period: usize, cci_period: usize, ma_type: &str, rank_limit: f64, max_active_pos_len: usize, profit_limit: f64, loss_limit: f64) -> Self {
        let origin_amount = init_cash / max_active_pos_len as f64;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            cci: CCI::new(cci_period, ma_type),
            vol_differ: Container::new(2),
            ranker: RollingRank::new(rank_period),
            rank_limit,
            entry_amount: origin_amount,
            max_pos_num: max_active_pos_len,
            available_pos_num: max_active_pos_len,
            profit_limit,
            loss_limit,
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        self.on_quote(bar);
        self.broker.update_portfolio_value(bar);
    }
}
