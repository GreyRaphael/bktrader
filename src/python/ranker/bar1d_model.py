from flaml import AutoML
from sklearn import metrics
import duckdb
import polars as pl
from scipy import signal
import datetime as dt


def prepare_savgol(window_length, polyorder=2) -> tuple:
    s_deriv1 = pl.Series(signal.savgol_coeffs(window_length, polyorder, deriv=1, delta=1, pos=window_length - 1, use="dot"))
    s_deriv2 = pl.Series(signal.savgol_coeffs(window_length, polyorder, deriv=2, delta=1, pos=window_length - 1, use="dot"))
    return s_deriv1, s_deriv2


def query_sector_codes(uri: str, sectors: list[int]) -> list[int]:
    placeholders = ",".join([str(s) for s in sectors])
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute(f"SELECT info.code FROM info JOIN bar1d ON info.code=bar1d.code WHERE info.sector IN ({placeholders}) AND bar1d.dt = (SELECT MAX(bar1d.dt) FROM bar1d)").fetchall()
    print("code length", len(records))
    return [record[0] for record in records]


def prepare_dataset(uri: str, start_dt: dt.date, end_dt: dt.date, codes: list[int], history_days=15, predict_days=5) -> pl.DataFrame:
    with duckdb.connect(uri, read_only=True) as con:
        placeholders = ",".join([str(c) for c in codes])
        df = (
            con.execute(
                f"""
    SELECT
        code,
        date_diff('day', DATE '1970-01-01', dt) as dti,
        amount / volume AS vwap,
        ln(vwap * adjfactor) AS adjvwap,
        ln(volume+1e-2) as adjvol, 
        -- dayofyear(dt) AS doy,
        -- dayofweek(dt) as dow,
        (vwap-low)/(high-low) AS boxp,
        vwap/open AS vdo,
        vwap/preclose AS vdp,
        open/preclose AS odp,
        GREATEST((high-low)/preclose, ABS(high/preclose-1), ABS(low/preclose-1)) AS tr,
        STDDEV(tr) OVER (PARTITION BY code ORDER BY dt ROWS BETWEEN 19 PRECEDING AND CURRENT ROW) AS tr_std20,
        turnover,
        LEAD(adjvwap, ?) OVER (PARTITION BY code ORDER BY dt) / adjvwap as ret{predict_days},
    FROM
        bar1d
    WHERE
        code IN ({placeholders}) AND dt BETWEEN ? AND ?
    ORDER BY
        code ASC, dt ASC
    """,
                [predict_days, start_dt, end_dt],
            )
            .pl()
            .drop_nans()
        )

    s_deriv1, s_deriv2 = prepare_savgol(history_days)
    return (
        df.with_columns(
            pl.col("adjvwap").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("vwap_deriv1"),
            pl.col("adjvwap").rolling_map(lambda s: s.dot(s_deriv2), window_size=history_days).over("code").alias("vwap_deriv2"),
            pl.col("adjvol").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("adjvol_deriv1"),
            pl.col("adjvol").rolling_map(lambda s: s.dot(s_deriv2), window_size=history_days).over("code").alias("adjvol_deriv2"),
            pl.col("boxp").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("boxp_deriv1"),
            pl.col("vdo").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("vdo_deriv1"),
            pl.col("vdp").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("vdp_deriv1"),
            pl.col("odp").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("odp_deriv1"),
            pl.col("tr").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("tr_deriv1"),
            pl.col("turnover").rolling_map(lambda s: s.dot(s_deriv1), window_size=history_days).over("code").alias("turnover_deriv1"),
            ((pl.col("ret5") - pl.min("ret5").over("dti")) / (pl.max("ret5").over("dti") - pl.min("ret5").over("dti")) * 30).round().cast(pl.UInt32).alias("label"),
        )
        .sort("dti")
        .select(pl.exclude("vwap", "adjvwap", "adjvol", f"ret{predict_days}", "code", "rank"))
        .drop_nulls()
        .filter(~pl.any_horizontal(pl.all().is_infinite()))
    )


def split_dataset(dataset: pl.DataFrame, split_date: dt.date):
    train_data = dataset.filter(pl.col("dti").cast(pl.Date) < split_date)
    test_data = dataset.filter(pl.col("dti").cast(pl.Date) >= split_date)
    train_df = train_data.select(pl.exclude("dti")).to_pandas()
    train_groups = train_data["dti"].to_pandas()
    X_test = test_data.select(pl.exclude("label", "dti")).to_pandas()
    y_test = test_data["label"].to_pandas()
    test_groups = test_data["dti"].to_pandas()
    return train_df, X_test, y_test, train_groups, test_groups


def print_importances(model):
    for name, loss in model.best_loss_per_estimator.items():
        print(f"{name} score: {1 - loss:.4f}")
    print(f"===>best model: {model.best_estimator}, train score: {1 - model.best_loss:.4f}")

    importances = {}
    importances_sum = 0
    for name, importance in zip(model.feature_names_in_.tolist(), model.feature_importances_.tolist()):
        importances[name] = importance
        importances_sum += importance
    for name, importance in sorted(importances.items(), key=lambda item: item[1], reverse=True):
        print(f"{name:10}: {importance / importances_sum:.4f}")
    print(f"===>time to find best model: {model.time_to_find_best_model:.2f} s")


if __name__ == "__main__":
    # end_dt = dt.date.today()
    end_dt = dt.date(2024, 9, 1)
    start_dt = end_dt.replace(year=end_dt.year - 2)
    offset = (end_dt - start_dt) * 0.7
    split_date = start_dt + offset
    print(f"train {(start_dt, split_date)}")
    print(f"test {(split_date, end_dt)}")

    codes = query_sector_codes("etf.db", [918, 1000056319000000, 1000056320000000, 1000056321000000, 1000056322000000])
    dataset = prepare_dataset("etf.db", start_dt, end_dt, codes, history_days=15, predict_days=5)
    train_df, X_test, y_test, train_groups, test_groups = split_dataset(dataset, split_date)
    print(f"train shape:{train_df.shape[0]}, test shape: {y_test.shape[0]}")

    model = AutoML()
    model.fit(dataframe=train_df, label="label", groups=train_groups, task="rank", time_budget=10, verbose=True)
    print_importances(model)

    y_pred = model.predict(X_test)
    print(f"==->test score: {metrics.ndcg_score([y_test], [y_pred]):.4f}")
