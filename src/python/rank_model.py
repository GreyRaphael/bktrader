from flaml import AutoML
from sklearn import metrics
import duckdb
import polars as pl
from scipy import signal
import datetime as dt

pl.Config.set_tbl_rows(20)


def prepare_savgol(window_length, polyorder=2) -> tuple:
    s_deriv1 = pl.Series(signal.savgol_coeffs(window_length, polyorder, deriv=1, delta=1, pos=window_length - 1, use="dot"))
    s_deriv2 = pl.Series(signal.savgol_coeffs(window_length, polyorder, deriv=2, delta=1, pos=window_length - 1, use="dot"))
    return s_deriv1, s_deriv2


def query_sector_codes(uri: str, sectors: list[int]) -> list[int]:
    placeholders = ",".join([str(s) for s in sectors])
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute(f"SELECT info.code FROM info JOIN bar1d ON info.code=bar1d.code WHERE info.sector IN ({placeholders}) AND bar1d.dt = (SELECT MAX(bar1d.dt) FROM bar1d)").fetchall()
    return [record[0] for record in records]


codes = query_sector_codes("etf.db", [918, 1000056319000000, 1000056320000000, 1000056321000000, 1000056322000000])
print("code length", len(codes))


def prepare_features(uri: str, codes: list[int], ret_days=5):
    with duckdb.connect(uri, read_only=True) as con:
        placeholders = ",".join([str(c) for c in codes])
        return (
            con.execute(
                f"""
    SELECT
        code,
        date_diff('day', DATE '1970-01-01', dt) as dti,
        amount / volume AS vwap,
        ln(vwap * adjfactor) AS adjvwap,
        ln(volume+1e-2) as adjvol, 
        dayofyear(dt) AS doy,
        dayofweek(dt) as dow,
        (vwap-low)/(high-low) AS boxp,
        vwap/open AS vdo,
        vwap/preclose AS vdp,
        open/preclose AS odp,
        GREATEST((high-low)/preclose, ABS(high/preclose-1), ABS(low/preclose-1)) AS tr,
        STDDEV(tr) OVER (PARTITION BY code ORDER BY dt ROWS BETWEEN 19 PRECEDING AND CURRENT ROW) AS tr_std20,
        turnover,
        LEAD(adjvwap, ?) OVER (PARTITION BY code ORDER BY dt) / adjvwap as ret{ret_days},
    FROM
        bar1d
    WHERE
        code IN ({placeholders})
    ORDER BY
        code ASC, dt ASC
    """,
                [ret_days],
            )
            .pl()
            .select(pl.exclude("vwap"))
            .drop_nulls()
        )


df = prepare_features("etf.db", codes)

win = 15
s_deriv1, s_deriv2 = prepare_savgol(win)
dfx = (
    df.with_columns(
        pl.col("adjvwap").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("vwap_deriv1"),
        pl.col("adjvwap").rolling_map(lambda s: s.dot(s_deriv2), window_size=win).over("code").alias("vwap_deriv2"),
        pl.col("adjvol").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("adjvol_deriv1"),
        pl.col("adjvol").rolling_map(lambda s: s.dot(s_deriv2), window_size=win).over("code").alias("adjvol_deriv2"),
        pl.col("boxp").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("boxp_deriv1"),
        pl.col("vdo").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("vdo_deriv1"),
        pl.col("vdp").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("vdp_deriv1"),
        pl.col("odp").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("odp_deriv1"),
        pl.col("tr").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("tr_deriv1"),
        pl.col("turnover").rolling_map(lambda s: s.dot(s_deriv1), window_size=win).over("code").alias("turnover_deriv1"),
        (pl.col("ret5").rank(method="ordinal") - 1).over("dti").alias("rank"),
    )
    .with_columns(
        (pl.col("rank") / pl.max("rank").over("dti") * 30).round().cast(pl.UInt32).alias("label"),
    )
    .sort("dti")
    .select(pl.exclude("adjvwap", "adjvol", "ret5", "code", "rank"))
    .drop_nulls()
    .filter(~pl.any_horizontal(pl.all().is_infinite()))
)


def split_dataset(dataset: pl.DataFrame, split_date: dt.date):
    train_data = dataset.filter(pl.col("dti").cast(pl.Date) < split_date)
    test_data = dataset.filter(pl.col("dti").cast(pl.Date) >= split_date)
    X_train = train_data.select(pl.exclude("label", "dti")).to_pandas()
    y_train = train_data["label"].to_pandas()
    train_groups_val = train_data.group_by("dti", maintain_order=True).len()["len"].to_pandas()
    X_test = test_data.select(pl.exclude("label", "dti")).to_pandas()
    y_test = test_data["label"].to_pandas()
    test_groups_val = test_data.group_by("dti", maintain_order=True).len()["len"].to_pandas()
    return X_train, X_test, y_train, y_test, train_groups_val, test_groups_val


split_date = dt.date(2024, 8, 1)
X_train, X_test, y_train, y_test, train_groups_val, test_groups_val = split_dataset(dfx, split_date)

model = AutoML()
model.fit(X_train, y_train, groups=train_groups_val, task="rank", time_budget=30, verbose=False)

print(f"train: {1 - model.best_loss:.4f}")
importances = {}
for name, importance in zip(model.feature_names_in_.tolist(), model.feature_importances_.tolist()):
    importances[name] = importance
for k, v in sorted(importances.items(), key=lambda item: item[1], reverse=True):
    print(f"{k:10}: {v:.4f}")

y_pred = model.predict(X_test)
metrics.ndcg_score([y_test], [y_pred])
