class BacktestEngine:
    def __init__(self, replayer, strategy):
        self.replayer = replayer
        self.strategy = strategy

    def run(self):
        for quote in self.replayer:
            self.strategy.on_update(quote)


class TradeEngine:
    def __init__(self, replayer, last_quote, strategy):
        self.replayer = replayer
        self.last_quote = last_quote
        self.strategy = strategy

    def run(self):
        for quote in self.replayer:
            self.strategy.on_update(quote)
        self.strategy.on_update(self.last_quote)


if __name__ == "__main__":
    import time
    import datetime as dt
    from quote.history import DuckdbReplayer
    from bktrader import strategy

    replayer = DuckdbReplayer(start=dt.date(2021, 1, 1), end=dt.date(2024, 11, 30), symbol=513650, uri="etf.db")
    # stg = strategy.DMAStrategy(init_cash=5e4)
    stg = strategy.GridPercent(init_cash=5e4, band_mult=2, max_active_pos_len=10)
    # stg = strategy.GridATR(init_cash=5e4, band_mult=1.5, max_active_pos_len=10)
    engine = BacktestEngine(replayer, stg)
    start_time = time.time()
    engine.run()
    print(f"cost {time.time()-start_time:.2f}s")
    print(f"last portfolio: {stg.broker.portfolio_value}")
    print(f"total_fees: {stg.broker.total_fees}")
    print(f"actives: {stg.broker.active_position_len()}")
