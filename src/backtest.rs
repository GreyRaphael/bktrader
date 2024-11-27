use std::fmt::Debug;

trait OnQuote<T> {
    fn on_quote(&self, item: &T);
}

#[derive(Debug)]
struct Bar {
    price: i32,
}

#[derive(Debug)]
struct Stock {
    symbol: String,
    price: f64,
}

struct MyRange<T> {
    data: Vec<T>,
}

impl<T> MyRange<T> {
    fn new(data: Vec<T>) -> Self {
        MyRange { data }
    }
}

impl<T> IntoIterator for MyRange<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

struct MyBarStrategy;

impl OnQuote<Bar> for MyBarStrategy {
    fn on_quote(&self, item: &Bar) {
        println!("{:?}", item);
    }
}

struct MyStockStrategy;

impl OnQuote<Stock> for MyStockStrategy {
    fn on_quote(&self, item: &Stock) {
        println!("{:?}", item);
    }
}

struct Engine<R, S, T> {
    range: R,
    strategy: S,
    _marker: std::marker::PhantomData<T>, // To store the item type
}

impl<R, S, T> Engine<R, S, T>
where
    R: IntoIterator<Item = T>,
    S: OnQuote<T>,
{
    fn new(range: R, strategy: S) -> Self {
        Engine {
            range,
            strategy,
            _marker: std::marker::PhantomData,
        }
    }

    // here must be self, cannot be &self or &mut self
    fn run(self) {
        for item in self.range.into_iter() {
            self.strategy.on_quote(&item);
        }
    }
}

// fn main() {
//     // Working with MyRange of Bar
//     let bar_range = MyRange::new(vec![Bar { price: 100 }, Bar { price: 200 }, Bar { price: 300 }]);
//     let bar_strategy = MyBarStrategy;
//     let bar_engine = Engine::new(bar_range, bar_strategy);
//     bar_engine.run();

//     // Working with MyRange of Stock
//     let stock_range = MyRange::new(vec![
//         Stock {
//             symbol: "AAPL".to_string(),
//             price: 145.67,
//         },
//         Stock {
//             symbol: "GOOGL".to_string(),
//             price: 2730.56,
//         },
//     ]);
//     let stock_strategy = MyStockStrategy;
//     let stock_engine = Engine::new(stock_range, stock_strategy);
//     stock_engine.run();
// }
