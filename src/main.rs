use duckdb::{params, Connection};
use rayon::prelude::*;

#[derive(Debug)]
pub struct Bar {
    pub code: u32,
    pub dt: i32,
    pub preclose: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub netvalue: f64,
    pub volume: f64,
    pub amount: f64,
    pub trades_count: f64,
    pub turnover: f64,
}

trait OnQuote<T> {
    fn on_quote(&self, item: &T);
}

struct MyBarStrategy;

impl OnQuote<Bar> for MyBarStrategy {
    fn on_quote(&self, item: &Bar) {
        println!("on_quote: {:?}", item);
    }
}

struct AnotherStrategy;

impl OnQuote<Bar> for AnotherStrategy {
    fn on_quote(&self, item: &Bar) {
        println!("AnotherStrategy received: {:?}", item);
    }
}

trait Engine {
    fn run(&self, code: &u32, start: &str, end: &str);
    fn run_multi(&self, codes: &[u32], start: &str, end: &str);
}

struct DuckBarEngine<T>
where
    T: OnQuote<Bar>,
{
    path: String,
    strategy: T,
}

impl<T> DuckBarEngine<T>
where
    T: OnQuote<Bar>,
{
    fn new(path: &str, strategy: T) -> Self {
        DuckBarEngine { path: path.into(), strategy }
    }
}

impl<T> Engine for DuckBarEngine<T>
where
    T: OnQuote<Bar> + std::marker::Sync,
{
    fn run(&self, code: &u32, start: &str, end: &str) {
        let conn = Connection::open(&self.path).expect("open error");
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
            .query_map(params![code, start, end], |row| {
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
            self.strategy.on_quote(&row.expect("row error"));
        }
    }

    fn run_multi(&self, codes: &[u32], start: &str, end: &str) {
        codes.into_par_iter().for_each(|code| {
            self.run(code, start, end);
        });
    }
}

fn main() {
    let stg = MyBarStrategy {};
    let replayer = DuckBarEngine::new("bar1d.db", stg);
    let codes = vec![510050, 513080, 513500];
    replayer.run_multi(&codes, "2024-11-20", "2024-12-31");

    let another_stg = AnotherStrategy {};
    let another_replayer = DuckBarEngine::new("bar1d.db", another_stg);
    another_replayer.run(&510050, "2024-11-20", "2024-12-31");
}
