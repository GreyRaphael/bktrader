use crate::datatype::{bar::Bar, position::Position, trade::Trade};
use pyo3::prelude::*;
use std::collections::VecDeque;

#[pyclass]
pub struct EtfBroker {
    pub init_cash: f64,
    #[pyo3(get)]
    pub cash: f64,
    #[pyo3(get)]
    pub portfolio_value: f64,
    ftc: f64,
    ptc: f64,
    positions: VecDeque<Position>,
    #[pyo3(get)]
    trades: Vec<Trade>,
    #[pyo3(get)]
    total_commission: f64,
}

impl EtfBroker {
    fn charge(&mut self, deal_amount: f64) -> f64 {
        let commission = self.ftc.max(deal_amount * self.ptc);
        self.total_commission += commission;
        commission
    }

    fn buy(&mut self, bar: &Bar, price: f64, volume: f64) {
        self.positions.push_back(Position {
            dt: bar.dt,
            price,
            volume,
        });
        let trade = Trade {
            code: bar.code,
            dt: bar.dt,
            price,
            volume,
        };
        println!("buy {:?}", trade);
        self.trades.push(trade);

        let deal_amount = price * volume;
        self.cash -= deal_amount + self.charge(deal_amount);
    }

    fn sell(&mut self, bar: &Bar, price: f64, volume: f64) {
        let mut remaining_vol = volume;
        let mut sold_vol = 0.0;

        while remaining_vol > 0.0 {
            if let Some(front) = self.positions.front_mut() {
                if front.volume > remaining_vol {
                    // Partial sell from the front position
                    front.volume -= remaining_vol;
                    sold_vol += remaining_vol;
                    remaining_vol = 0.0;
                } else {
                    // Sell entire front position
                    sold_vol += front.volume;
                    remaining_vol -= front.volume;
                    self.positions.pop_front();
                }
            } else {
                // No positions to sell; handle as needed (e.g., ignore, error, etc.)
                println!("Attempted to sell more vol than available.");
                break;
            }
        }

        let trade = Trade {
            code: bar.code,
            dt: bar.dt,
            price,
            volume: -1.0 * sold_vol,
        };
        println!("sell {:?}", trade);
        self.trades.push(trade);

        let deal_amount = price * sold_vol;
        self.cash += deal_amount - self.charge(deal_amount);
    }

    fn close_out(&mut self, bar: &Bar, price: f64) {
        // sell all
        let total_vol = self.positions_sum();
        self.sell(bar, price, total_vol);
    }
}

#[pymethods]
impl EtfBroker {
    #[new]
    #[pyo3(signature = (init_cash=5e4, ftc=5.0, ptc=1.5e-4))]
    pub fn new(init_cash: f64, ftc: f64, ptc: f64) -> Self {
        Self {
            init_cash,
            cash: init_cash,
            portfolio_value: init_cash,
            //  fixed transaction costs per trade (buy or sell)
            ftc,
            //  proportional transaction costs per trade (buy or sell)
            ptc,
            positions: VecDeque::with_capacity(10),
            trades: Vec::with_capacity(30),
            total_commission: 0.0,
        }
    }

    #[pyo3(signature = (bar, signal, price, volume=None, amount=None))]
    pub fn execute_order(
        &mut self,
        bar: &Bar,
        signal: u8,
        price: f64,
        volume: Option<f64>,
        amount: Option<f64>,
    ) {
        let order_vol = match (volume, amount) {
            (Some(vol), _) => vol,
            (None, Some(amt)) => amt / price,
            (None, None) => 0.0,
        };

        match signal {
            1 => self.buy(bar, price, order_vol),
            2 => self.sell(bar, price, order_vol),
            3 => self.close_out(bar, price),
            _ => {
                // hold
            }
        }

        self.portfolio_value = self.cash + self.positions_sum() * (bar.close as f64);
    }

    pub fn positions_front(&self) -> Option<Position> {
        self.positions.front().copied()
    }

    pub fn positions_back(&self) -> Option<Position> {
        self.positions.back().copied()
    }

    pub fn positions_len(&self) -> usize {
        self.positions.len()
    }

    pub fn positions_sum(&self) -> f64 {
        self.positions.iter().map(|pos| pos.volume).sum()
    }

    /// Get a list of all elements
    pub fn positions_list(&self) -> Vec<Position> {
        self.positions.iter().cloned().collect()
    }

    pub fn closed_position_num(&self) -> usize {
        let history_position_num = self.trades.iter().filter(|t| t.volume > 0.0).count();
        let opened_position_num = self.positions_len();
        let closed_position_num = history_position_num - opened_position_num;
        closed_position_num
    }

    pub fn profit_net(&self) -> f64 {
        self.portfolio_value / self.init_cash - 1.0
    }

    pub fn profit_gross(&self) -> f64 {
        self.profit_net() + self.loss_commission()
    }

    pub fn loss_net(&self) -> f64 {
        self.loss_gross() + self.loss_commission()
    }

    pub fn loss_gross(&self) -> f64 {
        // todo
        0.0
    }

    pub fn loss_commission(&self) -> f64 {
        self.total_commission / self.init_cash
    }
}
