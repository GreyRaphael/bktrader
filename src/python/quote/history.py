import datetime as dt
from bktrader import datatype


class DuckdbReplayer:
    def __init__(self, start: dt.date, end: dt.date, symbol: int, uri: str):
        # Try importing duckdb when needed.
        try:
            import duckdb
        except ImportError as e:
            raise ImportError("Class A requires 'duckdb'. Please install with: pip install duckdb") from e

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
            bar1d
        WHERE
            preclose IS NOT NULL
            AND code=?
            AND dt BETWEEN ? AND ?"""
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
                netvalue=record[7] or 0,
                volume=record[8],
                amount=record[9],
                trades_count=record[10],
                turnover=record[11],
            )


class DuckBatchReplayer:
    def __init__(self, start: dt.date, end: dt.date, codes: list[int], uri: str):
        try:
            import duckdb
        except ImportError as e:
            raise ImportError("Class A requires 'duckdb'. Please install with: pip install duckdb") from e

        self.conn = duckdb.connect(uri, read_only=True)
        placeholders = ",".join([str(c) for c in codes])
        query = f"""SELECT
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
            bar1d
        WHERE
            preclose IS NOT NULL
            AND code IN ({placeholders})
            AND dt BETWEEN ? AND ?
        ORDER BY
            code ASC, dt ASC"""
        self.conn.execute(query, [start, end])

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
                netvalue=record[7] or 0,
                volume=record[8],
                amount=record[9],
                trades_count=record[10],
                turnover=record[11],
            )


class PolarsReplayer:
    def __init__(self, start: dt.date, end: dt.date, symbol: int, uri: str):
        # Try importing polars when needed.
        try:
            import polars as pl
        except ImportError as e:
            raise ImportError("Class B requires 'polars'. Please install with: pip install polars") from e

        if uri.endswith(".ipc"):
            lazy_frame = pl.scan_ipc(uri)
        elif uri.endswith(".csv"):
            lazy_frame = pl.scan_csv(uri)
        elif uri.endswith(".parquet"):
            lazy_frame = pl.scan_parquet(uri)
        else:
            raise ValueError("Unsupported file extension")
        self.df = (
            lazy_frame.filter(pl.col("code") == symbol)
            .filter(pl.col("dt") >= start)
            .filter(pl.col("dt") <= end)
            .select(
                "code",
                pl.col("dt").cast(pl.UInt32),  # date since 1970-01-01
                (pl.col("preclose") * pl.col("adjfactor") / 1e4).round(3).alias("adj_preclose"),
                (pl.col("open") * pl.col("adjfactor") / 1e4).round(3).alias("adj_open"),
                (pl.col("high") * pl.col("adjfactor") / 1e4).round(3).alias("adj_high"),
                (pl.col("low") * pl.col("adjfactor") / 1e4).round(3).alias("adj_low"),
                (pl.col("close") * pl.col("adjfactor") / 1e4).round(3).alias("adj_close"),
                (pl.col("netvalue") * pl.col("adjfactor") / 1e4).round(3).alias("adj_netvalue"),
                "volume",
                (pl.col("amount") * pl.col("adjfactor") / 1e4).round(3).alias("amount"),
                pl.col("trades_count").fill_null(0),
                "turnover",
            )
            .collect()
        )

    def __iter__(self):
        return (
            datatype.Bar(
                code=row[0],
                dt=row[1],
                preclose=row[2],
                open=row[3],
                high=row[4],
                low=row[5],
                close=row[6],
                netvalue=row[7],
                volume=row[8],
                amount=row[9],
                trades_count=row[10],
                turnover=row[11],
            )
            for row in self.df.iter_rows()
        )


if __name__ == "__main__":
    # # DuckdbReplayer
    # duckdb_replayer = DuckdbReplayer(start=dt.date(2010, 1, 1), end=dt.date(2024, 11, 30), symbol=510050, uri="etf.db")
    # start_time = dt.datetime.now()
    # for bar in duckdb_replayer:
    #     print(bar)
    # time_elapsed = (dt.datetime.now() - start_time).total_seconds()
    # print(f"cost {time_elapsed:.2f}s")

    # test PolarsReplayer
    duckdb_replayer = PolarsReplayer(start=dt.date(2010, 1, 1), end=dt.date(2024, 11, 30), symbol=510050, uri="etf.ipc")
    start_time = dt.datetime.now()
    for bar in duckdb_replayer:
        print(bar)
    time_elapsed = (dt.datetime.now() - start_time).total_seconds()
    print(f"cost {time_elapsed:.2f}s")
