use crate::datatype::{bar::Bar, position::Position, trade::Trade};
use pyo3::prelude::*;
use std::collections::BTreeMap;

#[pyclass]
pub struct EtfBroker {
    pub init_cash: f64,
    #[pyo3(get)]
    pub cash: f64,
    #[pyo3(get)]
    pub portfolio_value: f64,
    ftc: f64,
    ptc: f64,
    active_positions: BTreeMap<u32, Position>,
    closed_positions: Vec<Position>,
    #[pyo3(get)]
    trades: Vec<Trade>,
    #[pyo3(get)]
    total_commission: f64,
    position_id: u32,
}

impl EtfBroker {
    fn charge(&mut self, deal_amount: f64) -> f64 {
        let commission = self.ftc.max(deal_amount * self.ptc);
        self.total_commission += commission;
        commission
    }

    fn buy(&mut self, bar: &Bar, price: f64, volume: f64) {
        self.active_positions
            .insert(0, Position::new(bar.dt, price, volume));
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
            if let Some((first_key, mut pos)) = self.active_positions.pop_first() {
                if pos.volume > remaining_vol {
                    // Partial sell from the front position
                    pos.volume -= remaining_vol;
                    sold_vol += remaining_vol;
                    remaining_vol = 0.0;
                    self.active_positions.insert(first_key, pos);
                } else {
                    // Sell entire front position
                    sold_vol += pos.volume;
                    remaining_vol -= pos.volume;
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
            active_positions: BTreeMap::new(),
            closed_positions: Vec::with_capacity(30),
            trades: Vec::with_capacity(30),
            total_commission: 0.0,
            position_id: 0,
        }
    }

    pub fn entry(&mut self, position: Position) {
        self.active_positions.insert(self.position_id, position);
    }

    pub fn exit(&mut self, position_ids: Vec<u32>, price: f64) {
        // Initialize the total sold volume
        let mut sold_vol = 0.0;

        // Iterate over position IDs to process each position
        for position_id in position_ids {
            // Remove the position from the active positions map
            if let Some(mut pos) = self.active_positions.remove(&position_id) {
                // Accumulate the volume of the sold position
                sold_vol += pos.volume;

                // Update the status of the position to 'closed'
                pos.status = 2;

                // Add the closed position to the closed_positions list
                self.closed_positions.push(pos);
            }
        }

        // Calculate the total deal amount based on the sold volume and price
        let deal_amount = price * sold_vol;

        // Update the cash balance after deducting any charges
        self.cash += deal_amount - self.charge(deal_amount);
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
        self.active_positions
            .first_key_value()
            .map(|(_, v)| v.clone())
    }

    pub fn positions_back(&self) -> Option<Position> {
        self.active_positions
            .last_key_value()
            .map(|(_, v)| v.clone())
    }

    pub fn positions_len(&self) -> usize {
        self.active_positions.len()
    }

    pub fn positions_sum(&self) -> f64 {
        self.active_positions.values().map(|pos| pos.volume).sum()
    }

    /// Get a list of all elements
    pub fn positions_list(&self) -> Vec<Position> {
        self.active_positions.values().cloned().collect()
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
