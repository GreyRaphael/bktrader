import datetime as dt
import polars as pl
from bktrader import datatype


class PolarsReplayer:
    def __init__(self, start: dt.date, end: dt.date, symbol: int, uri: str = "bar1d.ipc"):
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
    import time

    duckdb_replayer = PolarsReplayer(start=dt.date(2010, 1, 1), end=dt.date(2024, 11, 30), symbol=510050, uri="bar1d.ipc")
    start_time = time.time()
    for bar in duckdb_replayer:
        print(bar)
    print(f"cost {time.time()-start_time:.2f}s")
