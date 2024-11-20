// Define a Quote struct to represent market data
#[derive(Debug)]
struct Quote {
    price: f64,
}

// Define the StrategyBase trait with an on_quote method
trait StrategyBase {
    fn on_quote(&mut self, quote: &Quote);
}

// Define the ReplayerBase trait that provides an iterator over quotes
trait ReplayerBase {
    fn iter<'a>(&'a mut self) -> Box<dyn Iterator<Item = Quote> + 'a>;
}

// Implement the BacktestEngine struct that uses a replayer and a strategy
struct BacktestEngine<R, S>
where
    R: ReplayerBase,
    S: StrategyBase,
{
    replayer: R,
    strategy: S,
}

impl<R, S> BacktestEngine<R, S>
where
    R: ReplayerBase,
    S: StrategyBase,
{
    // Constructor for BacktestEngine
    fn new(replayer: R, strategy: S) -> Self {
        Self { replayer, strategy }
    }

    // The run method loops over the replayer and passes quotes to the strategy
    fn run(&mut self) {
        for quote in self.replayer.iter() {
            self.strategy.on_quote(&quote);
        }
    }
}

// Implement a simple replayer that replays a vector of quotes
struct SimpleReplayer {
    quotes: Vec<Quote>,
}

impl ReplayerBase for SimpleReplayer {
    fn iter<'a>(&'a mut self) -> Box<dyn Iterator<Item = Quote> + 'a> {
        // Use drain to consume the quotes vector and return an iterator
        Box::new(self.quotes.drain(..))
    }
}

// Implement a simple strategy that prints received quotes
struct SimpleStrategy;

impl StrategyBase for SimpleStrategy {
    fn on_quote(&mut self, quote: &Quote) {
        println!("Received quote: {:?}", quote);
    }
}

// // Main function to tie everything together
// fn main() {
//     // Sample quotes data
//     let quotes = vec![
//         Quote { price: 100.0 },
//         Quote { price: 101.5 },
//         Quote { price: 102.3 },
//     ];

//     // Create a replayer and strategy instance
//     let replayer = SimpleReplayer { quotes };
//     let strategy = SimpleStrategy;

//     // Initialize the backtest engine with the replayer and strategy
//     let mut engine = BacktestEngine::new(replayer, strategy);

//     // Run the backtest
//     engine.run();
// }
