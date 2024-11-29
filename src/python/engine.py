class BacktestEngine:
    def __init__(self, replayer, strategy):
        self.replayer = replayer
        self.strategy = strategy

    def run(self):
        for quote in self.replayer:
            self.strategy.on_bar(quote)


if __name__ == "__main__":
    import time
    import datetime as dt
    from replayer.duck import DuckdbReplayer
    from bktrader import strategy

    replayer = DuckdbReplayer(start=dt.date(2024, 1, 1), end=dt.date(2024, 11, 30), symbol=510050, uri="bar1d.db")
    stg = strategy.DMAStrategy()
    # stg = strategy.GridPercent()
    # stg = strategy.GridATR()
    engine = BacktestEngine(replayer, stg)
    start_time = time.time()
    engine.run()
    print(f"cost {time.time()-start_time:.2f}s")
