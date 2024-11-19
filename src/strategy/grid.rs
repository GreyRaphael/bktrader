use crate::broker::etf::EtfBroker;
use crate::datatype::bar::Bar;
use crate::ta::ma::MA;

use pyo3::prelude::*;

#[pyclass]
pub struct SimpleGrid {
    broker: EtfBroker,
    price_ma: MA,
    max_positions_len: usize,
    point_percent: f64,
    anti: bool,
    init_size: f64,
    size: f64,
    win: bool,
    mf: f64,
}

#[pymethods]
impl SimpleGrid {
    #[new]
    // ma_period 21 very good, type is sma
    pub fn new(init_cash: f64, ma_period: usize, ma_type: &str, max_positions_len: usize) -> Self {
        SimpleGrid {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            price_ma: MA::new(ma_period, ma_type),
            max_positions_len,
            point_percent: 0.02,
            anti: false,
            init_size: 1000.0,
            size: 1000.0,
            win: false,
            mf: 2.0,
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        let vwap = bar.amount / bar.volume;
        let baseline = self.price_ma.update(vwap);
        let upper = baseline * (1.0 + self.point_percent);
        let lower = baseline * (1.0 - self.point_percent);

        if self.broker.positions_len() >0 {

        }

        // no opened positions
        if self.broker.positions_len() < self.max_positions_len {
            if vwap> baseline {
                self.broker.execute_order(bar, 1, vwap, Some(self.size), None);
            }
        }

        if self.broker.positions_len() >0 {
            for pos in self.broker.positions_list() {
                if bar.high> pos.price {
                    // change pos.price to take_profit
                    self.broker.execute_order(bar, 2, pos.price, Some(pos.volume), None);
                }

                if bar.low < pos.price {
                    // change pos.price to stop_loss
                    self.broker.execute_order(bar, 2, pos.price, Some(pos.volume), None);
                }
            }
        }
    }
}
