import argparse
import polars as pl
import duckdb


def append_ipcs(ipc_path: str, db_uri: str):
    conn = duckdb.connect(db_uri, read_only=False)
    df_mapping = conn.execute("SELECT code,sector FROM bar1d WHERE dt=(SELECT MAX(dt) FROM bar1d)").pl()
    df_new = (
        pl.read_ipc(f"{ipc_path}/*.ipc")
        .join(df_mapping, on="code", how="left")
        .select("code", "dt", "preclose", "open", "high", "low", "close", "volume", "amount", "turnover", "netvalue", "trades_count", "adjfactor", "sector")
    )
    conn.execute("INSERT INTO bar1d SELECT * FROM df_new")
    conn.close()  # must close after save
    print(f"append ipc to {db_uri}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="往duckdb中追加数据")
    parser.add_argument("-indir", type=str, required=True, help="input dir contains ipcs")
    parser.add_argument("-db", type=str, default="bar1d.db", help="duckdb db path")

    args = parser.parse_args()
    append_ipcs(args.indir, args.db)
