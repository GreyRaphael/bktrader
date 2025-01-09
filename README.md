# bktrader

Trader with high-performance backtest engine in Rust

## rust sample

```rs
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

// Generic Engine struct
struct Engine<T> {
    uri: String,
    sql: String,
    code: u32,
    start: String,
    end: String,
    _marker: PhantomData<T>,
}

impl<T: FromRow> Engine<T> {
    fn new(uri: &str, sql: &str, code: u32, start: &str, end: &str) -> Self {
        Self {
            uri: uri.into(),
            sql: sql.into(),
            code,
            start: start.into(),
            end: end.into(),
            _marker: PhantomData,
        }
    }

    fn run<S: Strategy<T>>(&mut self, stg: &mut S) -> Result<(), duckdb::Error> {
        let conn = Connection::open(&self.uri)?;
        let mut stmt = conn.prepare(&self.sql)?;
        let rows = stmt.query_map(params![self.code, self.start, self.end], |row| T::from_row(row))?;

        for row in rows {
            let data = row?;
            stg.on_quote(&data);
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
        let mut stga = StrategyA;
        let mut engine = Engine::<Bar>::new(uri, bar_sql, code, start, end);
        if let Err(e) = engine.run(&mut stga) {
            eprintln!("Error processing Bar code {}: {:?}", code, e);
        }
    });

    let tick_sql = r#"
        SELECT
            code,
            ROUND(open * adjfactor / 1e4, 3) AS adj_open,
            ROUND(high * adjfactor / 1e4, 3) AS adj_high,
            ROUND(low * adjfactor / 1e4, 3) AS adj_low,
            ROUND(close * adjfactor / 1e4, 3) AS adj_close,
        FROM
            tick
        WHERE
            preclose IS NOT NULL
            AND code = ?
            AND dt BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)
    "#;

    code_list.par_iter().for_each(|&code| {
        let mut stgb = StrategyB;
        let mut engine = Engine::<Tick>::new(uri, tick_sql, code, start, end);
        if let Err(e) = engine.run(&mut stgb) {
            eprintln!("Error processing Tick code {}: {:?}", code, e);
        }
    });

    Ok(())
}
```

## idea

when applying the ranker model to 2000 stocks investment, I choose the `date` as group, for every group, I tag `output` as future 1 week return for every stocks with order, the input are 100 features for every stock, then train the model. Finally, for a new date, I take the new dataframe(2000x100) as input and predict the rank.

The approach you described is a practical and commonly used method in quantitative finance, particularly for stock ranking and portfolio construction. Here's a breakdown of how your approach aligns with industry practices and some considerations:

### Key Components of Your Approach:
1. **Grouping by Date**:  
   - Grouping by date is a standard practice in time-series cross-validation for financial data. It ensures that the model is trained on past data and tested on future data, preventing data leakage and maintaining the temporal order of the data.

2. **Target Variable (Future 1-Week Return)**:  
   - Using future returns as the target variable is a common way to train models for stock ranking. The ranking approach (e.g., XGBoost Ranker) is well-suited for this task because it focuses on the relative ordering of stocks rather than predicting exact return values.

3. **Features (100 Features per Stock)**:  
   - Using a large number of features (e.g., technical indicators, fundamental data, sentiment data, etc.) is typical in quantitative finance. Feature engineering and selection are critical to ensure the model captures meaningful patterns.

4. **Ranking Model (XGBoost Ranker)**:  
   - XGBoost Ranker is a popular choice for ranking tasks because it optimizes for pairwise comparisons, which aligns well with the goal of ranking stocks based on their expected future returns.

5. **Prediction for New Data**:  
   - Applying the trained model to new data (e.g., a new date's stock features) to predict ranks is a standard workflow. The predicted ranks can then be used to construct portfolios (e.g., long the top-ranked stocks and short the bottom-ranked stocks).

### Practical Considerations:
1. **Data Quality and Preprocessing**:  
   - Ensure that your features are properly normalized or standardized, and handle missing data appropriately. Financial data often contains noise, so robust preprocessing is essential.

2. **Feature Importance and Selection**:  
   - With 100 features, there is a risk of overfitting. Use techniques like feature importance analysis (provided by XGBoost) or dimensionality reduction (e.g., PCA) to select the most relevant features.

3. **Model Evaluation**:  
   - Use metrics like NDCG (Normalized Discounted Cumulative Gain) or Spearman's rank correlation to evaluate the model's ranking performance. Backtesting is also critical to assess the model's performance in a simulated trading environment.

4. **Overfitting and Regularization**:  
   - Financial data is noisy and non-stationary, so overfitting is a significant risk. Use techniques like early stopping, cross-validation, and regularization (e.g., L1/L2 regularization in XGBoost) to mitigate this.

5. **Temporal Dynamics**:  
   - Market conditions change over time, so the model's performance may degrade. Regularly retrain the model with new data and monitor its performance.

6. **Portfolio Construction**:  
   - The predicted ranks can be used to construct portfolios, but you need to consider transaction costs, liquidity, and risk management. For example, you might want to limit the number of stocks in your portfolio or use risk-adjusted performance metrics.

### Industry Applications:
- **Quantitative Hedge Funds**: Many hedge funds use similar ranking models to identify stocks with high expected returns for long/short strategies.
- **Robo-Advisors**: Some robo-advisors use ranking models to construct portfolios for clients.
- **Smart Beta Strategies**: Ranking models are often used in smart beta strategies to select stocks based on factors like value, momentum, or quality.

### Conclusion:
Your described approach is practical and aligns with industry practices. However, success in quantitative finance depends heavily on the quality of data, feature engineering, model evaluation, and risk management. Regular monitoring and adaptation of the model are essential to account for changing market conditions.