use bktrader::datatype::quote::Bar;
use duckdb::{params, Connection};
use rayon::prelude::*;
use std::marker::PhantomData;

// Define the mock Tick struct
#[derive(Debug)]
#[allow(dead_code)]
struct Tick {
    code: u32,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

// Define Strategies
struct StrategyA;
struct StrategyB;

trait Strategy<T> {
    fn on_quote(&mut self, quote: &T);
}

// Implement Strategy for StrategyA and StrategyB for Bar
impl Strategy<Bar> for StrategyA {
    fn on_quote(&mut self, quote: &Bar) {
        println!("StrategyA bar: {:?}", quote);
    }
}

impl Strategy<Bar> for StrategyB {
    fn on_quote(&mut self, quote: &Bar) {
        println!("StrategyB bar: {:?}", quote);
    }
}

// Implement Strategy for StrategyA and StrategyB for Tick
impl Strategy<Tick> for StrategyA {
    fn on_quote(&mut self, quote: &Tick) {
        println!("StrategyA tick: {:?}", quote);
    }
}

impl Strategy<Tick> for StrategyB {
    fn on_quote(&mut self, quote: &Tick) {
        println!("StrategyB tick: {:?}", quote);
    }
}

// Trait to map database rows to structs
trait FromRow {
    fn from_row(row: &duckdb::Row) -> Result<Self, duckdb::Error>
    where
        Self: Sized;
}

// Implement FromRow for Bar
impl FromRow for Bar {
    fn from_row(row: &duckdb::Row) -> Result<Self, duckdb::Error> {
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
    }
}

// Implement FromRow for Tick
impl FromRow for Tick {
    fn from_row(row: &duckdb::Row) -> Result<Self, duckdb::Error> {
        Ok(Tick {
            code: row.get(0)?,
            open: row.get(3)?,
            high: row.get(4)?,
            low: row.get(5)?,
            close: row.get(6)?,
        })
    }
}

// Define an enum to encapsulate all strategies// Define an enum to encapsulate all strategies
enum StrategyEnum {
    A(StrategyA),
    B(StrategyB),
    // Add more strategies here as needed
}

// Implement the Strategy trait for StrategyEnum for Bar
impl Strategy<Bar> for StrategyEnum {
    fn on_quote(&mut self, quote: &Bar) {
        match self {
            StrategyEnum::A(strategy_a) => strategy_a.on_quote(quote),
            StrategyEnum::B(strategy_b) => strategy_b.on_quote(quote),
            // Handle additional strategies here
        }
    }
}

// Implement the Strategy trait for StrategyEnum for Tick
impl Strategy<Tick> for StrategyEnum {
    fn on_quote(&mut self, quote: &Tick) {
        match self {
            StrategyEnum::A(strategy_a) => strategy_a.on_quote(quote),
            StrategyEnum::B(strategy_b) => strategy_b.on_quote(quote),
            // Handle additional strategies here
        }
    }
}

// Generic Engine struct
struct Engine<T> {
    uri: String,
    sql: String,
    strategies: StrategyEnum,
    code: u32,
    start: String,
    end: String,
    _marker: PhantomData<T>,
}

impl<T> Engine<T>
where
    T: FromRow,
    StrategyEnum: Strategy<T>,
{
    fn new(uri: &str, sql: &str, code: u32, start: &str, end: &str, stg: StrategyEnum) -> Self {
        Self {
            uri: uri.into(),
            sql: sql.into(),
            strategies: stg,
            code,
            start: start.into(),
            end: end.into(),
            _marker: PhantomData,
        }
    }

    fn run(&mut self) -> Result<(), duckdb::Error> {
        let conn = Connection::open(&self.uri)?;
        let mut stmt = conn.prepare(&self.sql)?;
        let rows = stmt.query_map(params![self.code, self.start, self.end], |row| T::from_row(row))?;

        for row in rows {
            let data = row?;
            self.strategies.on_quote(&data);
        }

        Ok(())
    }
}

fn main() -> Result<(), duckdb::Error> {
    // Define the database URI
    let uri = "etf.db";
    // Define the date range
    let start = "2024-11-20";
    let end = "2024-12-31";

    // List of codes to process
    let code_list: Vec<u32> = vec![510050, 513500, 159659];
    // use rayon to parallelize the processing
    // as code number is greater than strategy number, so parallelize the code list
    let bar_sql = r#"
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
            COALESCE(trades_count, 0) AS trades_count,
            turnover
        FROM
            bar1d
        WHERE
            preclose IS NOT NULL
            AND code = ?
            AND dt BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)
    "#;

    code_list.par_iter().for_each(|&code| {
        let stg = StrategyEnum::A(StrategyA);
        let mut engine = Engine::<Bar>::new(uri, bar_sql, code, start, end, stg);
        if let Err(e) = engine.run() {
            eprintln!("Error processing Bar code {}: {:?}", code, e);
        }
    });
    // code_list.par_iter().for_each(|&code| {
    //     let stg = StrategyEnum::B(StrategyB);
    //     let mut engine = Engine::<Tick>::new(uri, code, start, end, stg);
    //     if let Err(e) = engine.run() {
    //         eprintln!("Error processing Tick code {}: {:?}", code, e);
    //     }
    // });

    Ok(())
}
