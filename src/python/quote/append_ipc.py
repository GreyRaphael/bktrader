import argparse
import polars as pl
import duckdb


def append_ipcs(ipc_path: str, db_uri: str):
    df_new = pl.read_ipc(
        f"{ipc_path}/*.ipc",
        columns=["code", "dt", "preclose", "open", "high", "low", "close", "volume", "amount", "turnover", "netvalue", "trades_count", "adjfactor"],
    )
    with duckdb.connect(db_uri, read_only=False) as con:
        con.execute("INSERT INTO bar1d SELECT * FROM df_new")

    print(f"append ipc to {db_uri}, sample:")

    with duckdb.connect(db_uri, read_only=True) as con:
        df_sample = con.execute("SELECT * FROM bar1d WHERE code=(SELECT MAX(code) FROM bar1d)").pl()
        print(df_sample.tail(3))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="往duckdb中追加数据")
    parser.add_argument("-indir", type=str, required=True, help="input dir contains ipcs")
    parser.add_argument("-db", type=str, default="bar1d.db", help="duckdb db path")

    args = parser.parse_args()
    append_ipcs(args.indir, args.db)
