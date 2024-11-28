import datetime as dt
import duckdb
from bktrader import datatype


class DuckdbReplayer:
    def __init__(self, start: dt.date, end: dt.date, symbol: int, uri: str = "bar1d.db"):
        self.conn = duckdb.connect(uri, read_only=True)
        query = """SELECT
            code,
            date_diff('day', DATE '1970-01-01', dt) as days_since_epoch,
            ROUND(preclose * adjfactor / 1e4, 3) AS adj_preclose,
            ROUND(open * adjfactor / 1e4, 3) AS adj_open,
            ROUND(high * adjfactor / 1e4, 3) AS adj_high,
            ROUND(low * adjfactor / 1e4, 3) AS adj_low,
            ROUND(close * adjfactor / 1e4, 3) AS adj_close,
            ROUND(netvalue * adjfactor / 1e4, 3) AS adj_netvalue,
            volume,
            ROUND(amount * adjfactor / 1e4, 3) as adj_amount,
            -- handle null trades_count
            COALESCE(trades_count, 0) as trades_count,
            turnover,
        FROM
            etf
        WHERE
            code=? AND dt BETWEEN ? AND ?"""
        self.conn.execute(query, [symbol, start, end])

    def __iter__(self):
        return self

    def __next__(self) -> datatype.Bar:
        record = self.conn.fetchone()
        if record is None:
            self.conn.close()
            raise StopIteration
        else:
            return datatype.Bar(
                code=record[0],
                dt=record[1],
                preclose=record[2],
                open=record[3],
                high=record[4],
                low=record[5],
                close=record[6],
                netvalue=record[7],
                volume=record[8],
                amount=record[9],
                trades_count=record[10],
                turnover=record[11],
            )


if __name__ == "__main__":
    import time

    duckdb_replayer = DuckdbReplayer(start=dt.date(2010, 1, 1), end=dt.date(2024, 11, 30), symbol=510050, uri="bar1d.db")
    start_time = time.time()
    for bar in duckdb_replayer:
        print(bar)
    print(f"cost {time.time()-start_time:.2f}s")
