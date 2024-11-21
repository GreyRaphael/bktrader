use crate::broker::etf::EtfBroker;
use crate::datatype::bar::Bar;
use crate::ta::cross::Crosser;
use crate::ta::ma::{EMA, MA};
use pyo3::prelude::*;

#[pyclass]
pub struct SimpleGrid {
    broker: EtfBroker,
    base_ma: MA,
    max_active_pos_len: usize,
    band_mult: f64,
    entry_size: f64,
    premium_smooth_ma1: EMA,
    premium_smooth_ma2: EMA,
    premium_smooth_ma3: EMA,
    premium_smooth_ma4: EMA,
    premium_smooth_ma5: EMA,
    premium_smooth_ma6: EMA,
    premium_smooth_ma7: EMA,
    premium_smooth_ma8: EMA,
    discount_smooth_ma1: EMA,
    discount_smooth_ma2: EMA,
    discount_smooth_ma3: EMA,
    discount_smooth_ma4: EMA,
    discount_smooth_ma5: EMA,
    discount_smooth_ma6: EMA,
    discount_smooth_ma7: EMA,
    // discount_smooth_ma8: EMA,
    long_crox00: Crosser,
    long_crox01: Crosser,
    long_crox02: Crosser,
    long_crox03: Crosser,
    long_crox04: Crosser,
    long_crox05: Crosser,
    long_crox06: Crosser,
    long_crox07: Crosser,
    long_crox08: Crosser,
    long_crox09: Crosser,
    long_crox10: Crosser,
    long_crox11: Crosser,
    long_crox12: Crosser,
    long_crox13: Crosser,
    long_crox14: Crosser,
    short_crox00: Crosser,
    short_crox01: Crosser,
    short_crox02: Crosser,
    short_crox03: Crosser,
    short_crox04: Crosser,
    short_crox05: Crosser,
    short_crox06: Crosser,
    short_crox07: Crosser,
    short_crox08: Crosser,
    short_crox09: Crosser,
    short_crox10: Crosser,
    short_crox11: Crosser,
    short_crox12: Crosser,
    short_crox13: Crosser,
    short_crox14: Crosser,
    ids: [Option<u32>; 15],
}

#[pymethods]
impl SimpleGrid {
    #[new]
    #[pyo3(signature = (init_cash=5e5, ma_period=21, ma_type="sma", max_active_pos_len=6, band_mult=0.02))]
    pub fn new(init_cash: f64, ma_period: usize, ma_type: &str, max_active_pos_len: usize, band_mult: f64) -> Self {
        let original_size = (init_cash / max_active_pos_len as f64 / 100.0).floor() * 100.0;
        Self {
            broker: EtfBroker::new(init_cash, 5.0, 1.5e-4),
            base_ma: MA::new(ma_period, ma_type),
            max_active_pos_len,
            band_mult,
            entry_size: original_size,
            premium_smooth_ma1: EMA::new(5),
            premium_smooth_ma2: EMA::new(5),
            premium_smooth_ma3: EMA::new(5),
            premium_smooth_ma4: EMA::new(5),
            premium_smooth_ma5: EMA::new(5),
            premium_smooth_ma6: EMA::new(5),
            premium_smooth_ma7: EMA::new(5),
            premium_smooth_ma8: EMA::new(5),
            discount_smooth_ma1: EMA::new(5),
            discount_smooth_ma2: EMA::new(5),
            discount_smooth_ma3: EMA::new(5),
            discount_smooth_ma4: EMA::new(5),
            discount_smooth_ma5: EMA::new(5),
            discount_smooth_ma6: EMA::new(5),
            discount_smooth_ma7: EMA::new(5),
            // discount_smooth_ma8: EMA::new(5),
            long_crox00: Crosser::new(),
            long_crox01: Crosser::new(),
            long_crox02: Crosser::new(),
            long_crox03: Crosser::new(),
            long_crox04: Crosser::new(),
            long_crox05: Crosser::new(),
            long_crox06: Crosser::new(),
            long_crox07: Crosser::new(),
            long_crox08: Crosser::new(),
            long_crox09: Crosser::new(),
            long_crox10: Crosser::new(),
            long_crox11: Crosser::new(),
            long_crox12: Crosser::new(),
            long_crox13: Crosser::new(),
            long_crox14: Crosser::new(),
            short_crox00: Crosser::new(),
            short_crox01: Crosser::new(),
            short_crox02: Crosser::new(),
            short_crox03: Crosser::new(),
            short_crox04: Crosser::new(),
            short_crox05: Crosser::new(),
            short_crox06: Crosser::new(),
            short_crox07: Crosser::new(),
            short_crox08: Crosser::new(),
            short_crox09: Crosser::new(),
            short_crox10: Crosser::new(),
            short_crox11: Crosser::new(),
            short_crox12: Crosser::new(),
            short_crox13: Crosser::new(),
            short_crox14: Crosser::new(),
            ids: [None; 15],
        }
    }

    pub fn on_bar(&mut self, bar: &Bar) {
        let ohlc4 = (bar.open + bar.high + bar.low + bar.close) / 4.0;
        let vwap = bar.amount / bar.volume;
        let ma_center = self.base_ma.update(ohlc4);

        let premium_zone_1 = self.premium_smooth_ma1.update(ma_center * (1.0 + self.band_mult * 0.01));
        let premium_zone_2 = self.premium_smooth_ma2.update(ma_center * (1.0 + self.band_mult * 0.02));
        let premium_zone_3 = self.premium_smooth_ma3.update(ma_center * (1.0 + self.band_mult * 0.03));
        let premium_zone_4 = self.premium_smooth_ma4.update(ma_center * (1.0 + self.band_mult * 0.04));
        let premium_zone_5 = self.premium_smooth_ma5.update(ma_center * (1.0 + self.band_mult * 0.05));
        let premium_zone_6 = self.premium_smooth_ma6.update(ma_center * (1.0 + self.band_mult * 0.06));
        let premium_zone_7 = self.premium_smooth_ma7.update(ma_center * (1.0 + self.band_mult * 0.07));
        let premium_zone_8 = self.premium_smooth_ma8.update(ma_center * (1.0 + self.band_mult * 0.08));
        let discount_zone_1 = self.discount_smooth_ma1.update(ma_center * (1.0 - self.band_mult * 0.01));
        let discount_zone_2 = self.discount_smooth_ma2.update(ma_center * (1.0 - self.band_mult * 0.02));
        let discount_zone_3 = self.discount_smooth_ma3.update(ma_center * (1.0 - self.band_mult * 0.03));
        let discount_zone_4 = self.discount_smooth_ma4.update(ma_center * (1.0 - self.band_mult * 0.04));
        let discount_zone_5 = self.discount_smooth_ma5.update(ma_center * (1.0 - self.band_mult * 0.05));
        let discount_zone_6 = self.discount_smooth_ma6.update(ma_center * (1.0 - self.band_mult * 0.06));
        let discount_zone_7 = self.discount_smooth_ma7.update(ma_center * (1.0 - self.band_mult * 0.07));
        // let discount_zone_8 = self.discount_smooth_ma8.update(ma_center * (1.0 - self.band_mult * 0.08));

        if self.long_crox00.update(bar.low, discount_zone_7) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[0] = Some(pos_id);
        }
        if self.short_crox00.update(bar.high, discount_zone_6) == 1 {
            if let Some(pos_id) = self.ids[0] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox01.update(bar.low, discount_zone_6) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[1] = Some(pos_id);
        }
        if self.short_crox01.update(bar.high, discount_zone_5) == 1 {
            if let Some(pos_id) = self.ids[1] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox02.update(bar.low, discount_zone_5) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[2] = Some(pos_id);
        }
        if self.short_crox02.update(bar.high, discount_zone_4) == 1 {
            if let Some(pos_id) = self.ids[2] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox03.update(bar.low, discount_zone_4) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[3] = Some(pos_id);
        }
        if self.short_crox03.update(bar.high, discount_zone_3) == 1 {
            if let Some(pos_id) = self.ids[3] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox04.update(bar.low, discount_zone_3) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[4] = Some(pos_id);
        }
        if self.short_crox04.update(bar.high, discount_zone_2) == 1 {
            if let Some(pos_id) = self.ids[4] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox05.update(bar.low, discount_zone_2) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[5] = Some(pos_id);
        }
        if self.short_crox05.update(bar.high, discount_zone_1) == 1 {
            if let Some(pos_id) = self.ids[5] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox06.update(bar.low, discount_zone_1) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[6] = Some(pos_id);
        }
        if self.short_crox06.update(bar.high, ma_center) == 1 {
            if let Some(pos_id) = self.ids[6] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox07.update(bar.low, ma_center) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[7] = Some(pos_id);
        }
        if self.short_crox07.update(bar.high, premium_zone_1) == 1 {
            if let Some(pos_id) = self.ids[7] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox08.update(bar.low, premium_zone_1) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[8] = Some(pos_id);
        }
        if self.short_crox08.update(bar.high, premium_zone_2) == 1 {
            if let Some(pos_id) = self.ids[8] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox09.update(bar.low, premium_zone_2) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[9] = Some(pos_id);
        }
        if self.short_crox09.update(bar.high, premium_zone_3) == 1 {
            if let Some(pos_id) = self.ids[9] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox10.update(bar.low, premium_zone_3) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[10] = Some(pos_id);
        }
        if self.short_crox10.update(bar.high, premium_zone_4) == 1 {
            if let Some(pos_id) = self.ids[10] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox11.update(bar.low, premium_zone_4) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[11] = Some(pos_id);
        }
        if self.short_crox11.update(bar.high, premium_zone_5) == 1 {
            if let Some(pos_id) = self.ids[11] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox12.update(bar.low, premium_zone_5) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[12] = Some(pos_id);
        }
        if self.short_crox12.update(bar.high, premium_zone_6) == 1 {
            if let Some(pos_id) = self.ids[12] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox13.update(bar.low, premium_zone_6) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[13] = Some(pos_id);
        }
        if self.short_crox13.update(bar.high, premium_zone_7) == 1 {
            if let Some(pos_id) = self.ids[13] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }

        if self.long_crox14.update(bar.low, premium_zone_7) == -1 {
            let pos_id = self.broker.entry(bar, vwap, self.entry_size, None, None);
            self.ids[14] = Some(pos_id);
        }
        if self.short_crox14.update(bar.high, premium_zone_8) == 1 {
            if let Some(pos_id) = self.ids[14] {
                self.broker.exit(bar, vec![pos_id], vwap);
            }
        }
    }
}
