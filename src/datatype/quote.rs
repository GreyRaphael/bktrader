use pyo3::prelude::*;

#[pyclass]
#[derive(Debug)]
pub struct Bar {
    #[pyo3(get)]
    pub code: u32,
    #[pyo3(get)]
    pub dt: i32,
    #[pyo3(get)]
    pub preclose: f64,
    #[pyo3(get)]
    pub open: f64,
    #[pyo3(get)]
    pub high: f64,
    #[pyo3(get)]
    pub low: f64,
    #[pyo3(get)]
    pub close: f64,
    #[pyo3(get)]
    pub netvalue: f64,
    #[pyo3(get)]
    pub volume: f64,
    #[pyo3(get)]
    pub amount: f64,
    #[pyo3(get)]
    pub trades_count: f64,
    #[pyo3(get)]
    pub turnover: f64,
}

#[pymethods]
impl Bar {
    #[new]
    #[pyo3(signature = (code=0, dt=0, preclose=0.0, open=0.0, high=0.0, low=0.0, close=0.0, netvalue=0.0, volume=0.0, amount=0.0, trades_count=0.0, turnover=0.0))]
    fn new(code: u32, dt: i32, preclose: f64, open: f64, high: f64, low: f64, close: f64, netvalue: f64, volume: f64, amount: f64, trades_count: f64, turnover: f64) -> Self {
        Bar {
            code,
            dt,
            preclose,
            open,
            high,
            low,
            close,
            netvalue,
            volume,
            amount,
            trades_count,
            turnover,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass]
#[derive(Debug)]
pub struct BarM {
    #[pyo3(get)]
    pub code: u32,
    #[pyo3(get)]
    pub dt: i64,
    #[pyo3(get)]
    pub preclose: f64,
    #[pyo3(get)]
    pub open: f64,
    #[pyo3(get)]
    pub high: f64,
    #[pyo3(get)]
    pub low: f64,
    #[pyo3(get)]
    pub close: f64,
    #[pyo3(get)]
    pub volume: f64,
    #[pyo3(get)]
    pub amount: f64,
    #[pyo3(get)]
    pub trades_count: f64,
}

#[pymethods]
impl BarM {
    #[new]
    #[pyo3(signature = (code=0, dt=0, preclose=0.0, open=0.0, high=0.0, low=0.0, close=0.0,  volume=0.0, amount=0.0, trades_count=0.0))]
    fn new(code: u32, dt: i64, preclose: f64, open: f64, high: f64, low: f64, close: f64, volume: f64, amount: f64, trades_count: f64) -> Self {
        Self {
            code,
            dt,
            preclose,
            open,
            high,
            low,
            close,
            volume,
            amount,
            trades_count,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

// Tick
#[allow(dead_code)]
pub struct Tick {
    code: u32,
    dt: i64,
    preclose: f64,
    open: f64,
    last: f64,
    iopv: f64,
    high_limit: f64,
    low_limit: f64,
    trades_count: f64,
    volume: f64,
    tot_ask_volume: f64,
    tot_bid_volume: f64,
    amount: f64,
    avg_ask_price: f64,
    avg_bid_price: f64,
    ask_prices: [f64; 10],
    bid_prices: [f64; 10],
    ask_volumes: [f64; 10],
    bid_volumes: [f64; 10],
    ask_nums: [f64; 10],
    bid_nums: [f64; 10],
}

// Order
#[allow(dead_code)]
pub struct Order {
    code: u32,
    dt: i64, // datetime
    seq_no: u64,
    price: f64,
    volume: f64,
    bs_flag: u8,
    order_type: u8,
    origin_seq_no: u64,
}

// Trade
#[allow(dead_code)]
pub struct Trade {
    code: u32,
    dt: i64,
    seq_no: u64,
    price: f64,
    volume: f64,
    bs_flag: u8,
    ask_seq_no: u64,
    bid_seq_no: u64,
}
