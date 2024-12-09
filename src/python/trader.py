class TradeEngine:
    def __init__(self, replayer, last_quote, strategy):
        self.replayer = replayer
        self.last_quote = last_quote
        self.strategy = strategy

    def run(self):
        for quote in self.replayer:
            self.strategy.on_bar(quote)
        self.strategy.on_bar(self.last_quote)

if __name__ == "__main__":
    pass