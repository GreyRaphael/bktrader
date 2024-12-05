use super::analyzer::Analyzer;
use crate::datatype::{quote::Bar, position::Position, position::PositionStatus};
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
#[derive(Clone)] // for the #[pyo3(get)] in strategies
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
    total_fees: f64,
    #[pyo3(get)]
    analyzer: Analyzer,
}

impl EtfBroker {
    fn charge(&mut self, deal_amount: f64) -> f64 {
        let fees = self.ftc.max(deal_amount * self.ptc);
        self.total_fees += fees;
        fees
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
            total_fees: 0.0,
            analyzer: Analyzer::new(),
        }
    }

    #[pyo3(signature = (bar, price, volume, stop_loss=None, take_profit=None))]
    pub fn entry(&mut self, bar: &Bar, price: f64, volume: f64, stop_loss: Option<f64>, take_profit: Option<f64>) -> u32 {
        let deal_amount = price * volume;
        let fees = self.charge(deal_amount);
        self.cash -= deal_amount + fees;

        // open position
        let mut pos = Position::new(bar.dt, price, volume);
        pos.fees = fees;
        pos.stop_loss = stop_loss;
        pos.take_profit = take_profit;
        // println!("entry {:?}", pos);

        self.positions.push(pos);

        // return position id
        pos.id
    }

    pub fn exit(&mut self, bar: &Bar, position_ids: Vec<u32>, price: f64) {
        // position_id: index mapping in all positions
        let position_map: HashMap<u32, usize> = self.positions.iter().enumerate().map(|(i, pos)| (pos.id, i)).collect();

        let mut sold_vol = 0.0;
        let mut indices_to_update = Vec::with_capacity(position_ids.len());
        for id in position_ids {
            if let Some(&index) = position_map.get(&id) {
                let position = &mut self.positions[index];
                position.status = PositionStatus::Closed;
                position.exit_dt = Some(bar.dt);
                position.exit_price = Some(price);
                sold_vol += position.volume;
                indices_to_update.push(index);
            }
        }

        // Calculate deal amount and fees
        let deal_amount = price * sold_vol;
        let fees = self.charge(deal_amount);
        self.cash += deal_amount - fees;

        // Calculate average fees
        let avg_fees = if !indices_to_update.is_empty() { fees / indices_to_update.len() as f64 } else { 0.0 };

        // Update fees and PnL for each position
        for &index in &indices_to_update {
            let position = &mut self.positions[index];
            position.fees = avg_fees;
            position.fees += fees;
            position.pnl = Some((price - position.entry_price) * position.volume);
            // println!("exit {:?}", position);
        }
    }

    pub fn update_portfolio_value(&mut self, bar: &Bar) {
        self.portfolio_value = self.cash + self.active_positions_sum() * bar.close;
        self.analyzer.update(self.portfolio_value);
    }

    pub fn update_active_pnl(&mut self, bar: &Bar) {
        self.positions.iter_mut().filter(|pos| pos.status == PositionStatus::Opened).for_each(|pos| {
            pos.pnl = Some((bar.close - pos.entry_price) * pos.volume);
        });
    }

    pub fn active_position_first(&self) -> Option<Position> {
        self.positions.iter().find(|pos| pos.status == PositionStatus::Opened).copied()
    }

    pub fn active_position_last(&self) -> Option<Position> {
        self.positions.iter().rfind(|pos| pos.status == PositionStatus::Opened).copied()
    }

    pub fn active_position_len(&self) -> usize {
        self.positions.iter().filter(|pos| pos.status == PositionStatus::Opened).count()
    }

    pub fn active_positions_sum(&self) -> f64 {
        let mut sum = 0.0;
        for pos in &self.positions {
            if pos.status == PositionStatus::Opened {
                sum += pos.volume;
            }
        }
        sum
    }

    pub fn active_positions(&self) -> Vec<Position> {
        self.positions.iter().filter(|pos| pos.status == PositionStatus::Opened).copied().collect()
    }

    pub fn closed_positions(&self) -> Vec<Position> {
        self.positions.iter().filter(|pos| pos.status == PositionStatus::Closed).copied().collect()
    }

    pub fn profit_net(&self) -> f64 {
        self.portfolio_value / self.init_cash - 1.0
    }

    pub fn profit_gross(&self) -> f64 {
        self.profit_net() + self.loss_fees()
    }

    pub fn loss_net(&self) -> f64 {
        self.loss_gross() + self.loss_fees()
    }

    pub fn loss_gross(&self) -> f64 {
        // todo
        0.0
    }

    pub fn loss_fees(&self) -> f64 {
        self.total_fees / self.init_cash
    }
}
