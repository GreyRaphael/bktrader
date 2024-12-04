use bktrader::{datatype::bar::Bar, strategy::base::QuoteHandler};
use duckdb::{params, Connection};

#[derive(Debug)]
struct Tick {
    code: u32,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

struct StrategyA;

impl QuoteHandler<Bar> for StrategyA {
    fn on_quote(&mut self, quote: &Bar) {
        println!("StrategyA bar: {:?}", quote);
    }
}

impl QuoteHandler<Tick> for StrategyA {
    fn on_quote(&mut self, quote: &Tick) {
        println!("StrategyA tick: {:?}", quote);
    }
}

struct StrategyB;

impl QuoteHandler<Bar> for StrategyB {
    fn on_quote(&mut self, quote: &Bar) {
        println!("StrategyB bar: {:?}", quote);
    }
}

impl QuoteHandler<Tick> for StrategyB {
    fn on_quote(&mut self, quote: &Tick) {
        println!("StrategyB tick: {:?}", quote);
    }
}

enum BarStrategy {
    Simple(StrategyA),
    Complex(StrategyB),
}

impl BarStrategy {
    fn on_bar(&mut self, quote: &Bar) {
        match self {
            Self::Simple(simple) => simple.on_quote(quote),
            Self::Complex(complex) => complex.on_quote(quote),
        }
    }
}

struct BarEngine {
    uri: String,
    strategy: BarStrategy,
    code: u32,
    start: String,
    end: String,
}

impl BarEngine {
    fn new(uri: &str, strategy: BarStrategy, code: u32, start: &str, end: &str) -> Self {
        Self {
            uri: uri.into(),
            strategy,
            code,
            start: start.into(),
            end: end.into(),
        }
    }

    fn run(&mut self) {
        let conn = Connection::open(&self.uri).expect("open error");
        let query = r#"
        SELECT
            code,
            date_diff('day', DATE '1970-01-01', dt) AS days_since_epoch,
            ROUND(preclose * adjfactor / 1e4, 3) AS adj_preclose,
            ROUND(open * adjfactor / 1e4, 3) AS adj_open,
            ROUND(high * adjfactor / 1e4, 3) AS adj_high,
            ROUND(low * adjfactor / 1e4, 3) AS adj_low,
            ROUND(close * adjfactor / 1e4, 3) AS adj_close,
            ROUND(netvalue * adjfactor / 1e4, 3) AS adj_netvalue,
            volume,
            ROUND(amount * adjfactor / 1e4, 3) AS adj_amount,
            -- Handle null trades_count
            COALESCE(trades_count, 0) AS trades_count,
            turnover
        FROM
            etf
        WHERE
            preclose IS NOT NULL
            AND code = ?
            AND dt BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)
    "#;

        let mut stmt = conn.prepare(&query).expect("query error");
        let rows = stmt
            .query_map(params![self.code, self.start, self.end], |row| {
                Ok(Bar {
                    code: row.get(0)?,
                    dt: row.get(1)?,
                    preclose: row.get(2)?,
                    open: row.get(3)?,
                    high: row.get(4)?,
                    low: row.get(5)?,
                    close: row.get(6)?,
                    netvalue: row.get(7)?,
                    volume: row.get(8)?,
                    amount: row.get(9)?,
                    trades_count: row.get(10)?,
                    turnover: row.get(11)?,
                })
            })
            .expect("parse error");

        // Collect the results into a vector
        for row in rows {
            self.strategy.on_bar(&row.expect("row error"));
        }
    }
}

fn main() {
    // same strategy for diffrent codes
    let code_list: Vec<u32> = vec![510050, 513500, 159659];
    let mut engine_list = Vec::with_capacity(3);
    for code in code_list {
        let stg = BarStrategy::Simple(StrategyA {});
        let engine = BarEngine::new("bar1d.db", stg, code, "2024-11-20", "2024-12-31");
        engine_list.push(engine);
    }
    for engine in engine_list.iter_mut() {
        engine.run();
    }

    // same code for diffrent strategy
    let strategy_list = vec![BarStrategy::Simple(StrategyA {}), BarStrategy::Complex(StrategyB {})];
    let mut engine_list = Vec::with_capacity(2);
    for stg in strategy_list {
        let engine = BarEngine::new("bar1d.db", stg, 510050, "2024-11-20", "2024-12-31");
        engine_list.push(engine);
    }
    for engine in engine_list.iter_mut() {
        engine.run();
    }
}
