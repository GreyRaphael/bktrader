use super::base::OnQuote;
use crate::broker::etf::EtfBroker;
use crate::datatype::bar::Bar;
use crate::ta::cross::Crosser;
use crate::ta::ma::{EMA, MA};
use crate::ta::volatility::ATR;
use pyo3::prelude::*;

#[pyclass]
pub struct GridPercent {
    broker: EtfBroker,
    base_ma: MA,
    available_pos_num: usize,
    band_mult: f64,
    entry_size: f64,
    premium_smooth_mas: Vec<EMA>,
    discount_smooth_mas: Vec<EMA>,
    long_croxes: Vec<Crosser>,
    short_croxes: Vec<Crosser>,
    ids: Vec<Option<u32>>,
    entry_zones: Vec<f64>,
    exit_zones: Vec<f64>,
}

impl OnQuote<Bar> for GridPercent {
    fn on_quote(&mut self, bar: &Bar) {
        let ohlc4 = (bar.open + bar.high + bar.low + bar.close) / 4.0;
        let vwap = bar.amount / bar.volume;
        let ma_center = self.base_ma.update(ohlc4);

        let premium_zones: Vec<f64> = (0..8)
            .map(|i| self.premium_smooth_mas[i].update(ma_center * (1.0 + 0.01 * self.band_mult * (i as f64 + 1.0))))
            .collect();
        let discount_zones: Vec<f64> = (0..8)
            .map(|i| self.discount_smooth_mas[i].update(ma_center * (1.0 - 0.01 * self.band_mult * (i as f64 + 1.0))))
            .collect();

        // entry zones
        for i in 0..=7 {
            self.entry_zones[i] = discount_zones[7 - i];
        }
        self.entry_zones[8] = ma_center;
        for i in 9..16 {
            self.entry_zones[i] = premium_zones[i - 9];
        }
        // exit zones
        for i in 0..=6 {
            self.exit_zones[i] = discount_zones[6 - i];
        }
        self.exit_zones[7] = ma_center;
        for i in 8..16 {
            self.exit_zones[i] = premium_zones[i - 8];
        }

        // Accumulate exit postion ids
        // exit should before entry
        let mut positions_to_exit = Vec::new();
        for i in 0..16 {
            if self.short_croxes[i].update(bar.high, self.exit_zones[i]) == 1 {
                if let Some(pos_id) = self.ids[i] {
                    positions_to_exit.push(pos_id);
                    self.available_pos_num += 1;
                    self.ids[i] = None;
                }
            }
        }
        if !positions_to_exit.is_empty() {
            self.broker.exit(bar, positions_to_exit, vwap);
        }

        // if opened postions smaller than threshold, entry position; else no entry
        if self.available_pos_num > 0 {
            // Find the deepest entry crossing and accumulate entry size
            let mut deepest_entry_crossing = None;
            let mut total_entry_size = 0.0;
            for i in 15..=0 {
                if self.long_croxes[i].update(bar.low, self.entry_zones[i]) == -1 {
                    total_entry_size += self.entry_size;
                    self.available_pos_num -= 1;
                    deepest_entry_crossing = Some(i);
                }
            }
            if let Some(i) = deepest_entry_crossing {
                let pos_id = self.broker.entry(bar, vwap, total_entry_size, None, None);
                self.ids[i] = Some(pos_id);
            }
        }
    }
}

#[pymethods]
impl GridPercent {
    #[new]
    #[pyo3(signature = (init_cash=5e5, ma_period=21, ma_type="sma", max_active_pos_len=6, band_mult=0.02))]
    pub fn new(init_cash: f64, ma_period: usize, ma_type: &str, max_active_pos_len: usize, band_mult: f64) -> Self {
        let original_size = (init_cash / max_active_pos_len as f64 / 100.0).floor() * 100.0;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            base_ma: MA::new(ma_period, ma_type),
            available_pos_num: max_active_pos_len,
            band_mult,
            entry_size: original_size,
            premium_smooth_mas: (0..8).map(|_| EMA::new(5)).collect(),
            discount_smooth_mas: (0..8).map(|_| EMA::new(5)).collect(),
            long_croxes: (0..16).map(|_| Crosser::new()).collect(),
            short_croxes: (0..16).map(|_| Crosser::new()).collect(),
            ids: vec![None; 16],
            entry_zones: vec![0.0; 16],
            exit_zones: vec![0.0; 16],
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        self.on_quote(bar);
    }
}

#[pyclass]
pub struct GridATR {
    broker: EtfBroker,
    base_ma: MA,
    atr: ATR,
    band_mult: f64,
    available_pos_num: usize,
    entry_size: f64,
    premium_smooth_mas: Vec<EMA>,
    discount_smooth_mas: Vec<EMA>,
    long_croxes: Vec<Crosser>,
    short_croxes: Vec<Crosser>,
    ids: Vec<Option<u32>>,
    entry_zones: Vec<f64>,
    exit_zones: Vec<f64>,
}

impl OnQuote<Bar> for GridATR {
    fn on_quote(&mut self, bar: &Bar) {
        let ohlc4 = (bar.open + bar.high + bar.low + bar.close) / 4.0;
        let vwap = bar.amount / bar.volume;
        let ma_center = self.base_ma.update(ohlc4);
        let atr_val = self.atr.update(bar.high, bar.low, bar.preclose);

        let premium_zones: Vec<f64> = (0..8).map(|i| self.premium_smooth_mas[i].update(ma_center + self.band_mult * atr_val * (i as f64 + 1.0))).collect();
        let discount_zones: Vec<f64> = (0..8).map(|i| self.discount_smooth_mas[i].update(ma_center + self.band_mult * atr_val * (i as f64 + 0.1))).collect();

        // entry zones
        for i in 0..=7 {
            self.entry_zones[i] = discount_zones[7 - i];
        }
        self.entry_zones[8] = ma_center;
        for i in 9..16 {
            self.entry_zones[i] = premium_zones[i - 9];
        }
        // exit zones
        for i in 0..=6 {
            self.exit_zones[i] = discount_zones[6 - i];
        }
        self.exit_zones[7] = ma_center;
        for i in 8..16 {
            self.exit_zones[i] = premium_zones[i - 8];
        }

        // Accumulate exit postion ids
        // exit should before entry
        let mut positions_to_exit = Vec::new();
        for i in 0..16 {
            if self.short_croxes[i].update(bar.high, self.exit_zones[i]) == 1 {
                if let Some(pos_id) = self.ids[i] {
                    positions_to_exit.push(pos_id);
                    self.available_pos_num += 1;
                    self.ids[i] = None;
                }
            }
        }
        if !positions_to_exit.is_empty() {
            self.broker.exit(bar, positions_to_exit, vwap);
        }

        // if opened postions smaller than threshold, entry position; else no entry
        if self.available_pos_num > 0 {
            // Find the deepest entry crossing and accumulate entry size
            let mut deepest_entry_crossing = None;
            let mut total_entry_size = 0.0;
            for i in 15..=0 {
                if self.long_croxes[i].update(bar.low, self.entry_zones[i]) == -1 {
                    total_entry_size += self.entry_size;
                    self.available_pos_num -= 1;
                    deepest_entry_crossing = Some(i);
                }
            }
            if let Some(i) = deepest_entry_crossing {
                let pos_id = self.broker.entry(bar, vwap, total_entry_size, None, None);
                self.ids[i] = Some(pos_id);
            }
        }
    }
}

#[pymethods]
impl GridATR {
    #[new]
    #[pyo3(signature = (init_cash=5e5, ma_period=21, ma_type="sma", atr_period=60, atr_ma_type="rma", max_active_pos_len=6, band_mult=0.02))]
    pub fn new(init_cash: f64, ma_period: usize, ma_type: &str, atr_period: usize, atr_ma_type: &str, max_active_pos_len: usize, band_mult: f64) -> Self {
        let original_size = (init_cash / max_active_pos_len as f64 / 100.0).floor() * 100.0;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            base_ma: MA::new(ma_period, ma_type),
            atr: ATR::new(atr_period, atr_ma_type),
            band_mult,
            available_pos_num: max_active_pos_len,
            entry_size: original_size,
            premium_smooth_mas: (0..8).map(|_| EMA::new(5)).collect(),
            discount_smooth_mas: (0..8).map(|_| EMA::new(5)).collect(),
            long_croxes: (0..16).map(|_| Crosser::new()).collect(),
            short_croxes: (0..16).map(|_| Crosser::new()).collect(),
            ids: vec![None; 16],
            entry_zones: vec![0.0; 16],
            exit_zones: vec![0.0; 16],
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        self.on_quote(bar);
    }
}
