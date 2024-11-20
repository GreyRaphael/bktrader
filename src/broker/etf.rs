use crate::datatype::{bar::Bar, position::Position};
use pyo3::prelude::*;

#[pyclass]
pub struct EtfBroker {
    pub init_cash: f64,
    #[pyo3(get)]
    pub cash: f64,
    #[pyo3(get)]
    pub portfolio_value: f64,
    ftc: f64,
    ptc: f64,
    #[pyo3(get)]
    pub positions: Vec<Position>,
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
            positions: Vec::with_capacity(100),
            total_commission: 0.0,
            position_id: 0,
        }
    }

    #[pyo3(signature = (bar, price, volume, stop_loss=None, take_profit=None))]
    pub fn entry(
        &mut self,
        bar: &Bar,
        price: f64,
        volume: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) {
        self.position_id += 1;
        let pos = Position {
            id: self.position_id,
            entry_dt: bar.dt,
            exit_dt: None,
            entry_price: price,
            exit_price: None,
            stop_loss,
            take_profit,
            status: 1,
            volume,
        };
        println!("entry {:?}", pos);
        self.positions.push(pos);

        let deal_amount = price * volume;
        self.cash -= deal_amount + self.charge(deal_amount);
    }

    pub fn exit(&mut self, bar: &Bar, position_ids: Vec<u32>, price: f64) {
        let mut sold_vol = 0.0;

        for id in position_ids {
            if let Some(position) = self.positions.iter_mut().find(|pos| pos.id == id) {
                position.status = 2;
                position.exit_dt = Some(bar.dt);
                position.exit_price = Some(price);
                sold_vol += position.volume;
            };
        }

        let deal_amount = price * sold_vol;
        self.cash += deal_amount - self.charge(deal_amount);

        // self.portfolio_value = self.cash + self.positions_sum() * (bar.close as f64);
    }

    pub fn active_position_first(&self) -> Option<Position> {
        self.positions.iter().find(|pos| pos.status == 1).copied()
    }

    pub fn active_position_last(&self) -> Option<Position> {
        self.positions.iter().rfind(|pos| pos.status == 1).copied()
    }

    pub fn active_position_len(&self) -> usize {
        self.positions.iter().filter(|pos| pos.status == 1).count()
    }

    pub fn positions_sum(&self) -> f64 {
        self.positions.iter().map(|pos| pos.volume).sum()
    }

    pub fn active_positions(&self) -> Vec<Position> {
        self.positions
            .iter()
            .filter(|pos| pos.status == 1)
            .copied()
            .collect::<Vec<_>>()
    }

    pub fn closed_positions(&self) -> Vec<Position> {
        self.positions
            .iter()
            .filter(|pos| pos.status == 2)
            .copied()
            .collect::<Vec<_>>()
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
