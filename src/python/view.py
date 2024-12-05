import altair as alt
import datetime as dt
import polars as pl
import duckdb
from bktrader import datatype
from replayer.duck import DuckdbReplayer
from engine import BacktestEngine


def calc_ls_chart(positions: list[datatype.Bar]):
    opened_list = [(pos.entry_dt, pos.id, pos.entry_price, pos.volume) for pos in positions]
    closed_list = [(pos.exit_dt, pos.id, pos.exit_price, pos.volume, pos.pnl, pos.fees) for pos in positions if pos.pnl is not None]
    df_opened = pl.from_records(opened_list, orient="row", schema=["dt", "id", "price", "volume"]).with_columns(pl.from_epoch("dt", time_unit="d"))
    df_closed = pl.from_records(closed_list, orient="row", schema=["dt", "id", "price", "volume", "pnl", "fees"]).with_columns(pl.from_epoch("dt", time_unit="d"))

    long_base = alt.Chart(df_opened).encode(alt.X("dt:T").axis(format="%Y-%m-%d", labelAngle=-45), tooltip=["dt", "id", "price", "volume"])
    long_markers = long_base.mark_point(shape="triangle-up", color="blue", yOffset=20).encode(y="price")
    long_texts = long_base.mark_text(align="center", baseline="top", yOffset=35, color="blue").encode(y="price", text="id")

    short_base = (
        alt.Chart(df_closed)
        .transform_window(
            cumulative_count="count()",
            groupby=["dt", "price"],
        )
        .encode(
            alt.X("dt:T").axis(format="%Y-%m-%d", labelAngle=-45),
            tooltip=["dt", "id", "volume", "price", "pnl", "fees"],
        )
    )
    short_markers = short_base.mark_point(shape="triangle-down", color="brown", yOffset=-20).encode(y="price")
    short_texts = short_base.mark_text(align="center", baseline="bottom", dy=alt.expr("-10*datum.cumulative_count-15"), color="brown").encode(y="price", text="id")
    return long_markers + long_texts + short_markers + short_texts


def calc_candle_chart(uri: str, code: int, start: dt.date, end: dt.date):
    query = """
    SELECT
        dt,
        ROUND(open * adjfactor / 1e4, 3) AS adj_open,
        ROUND(high * adjfactor / 1e4, 3) AS adj_high,
        ROUND(low * adjfactor / 1e4, 3) AS adj_low,
        ROUND(close * adjfactor / 1e4, 3) AS adj_close,
    FROM
        etf
    WHERE 
        code = ? AND dt BETWEEN ? AND ?
    """

    with duckdb.connect(uri) as conn:
        df = conn.execute(query, [code, start, end]).pl()

    open_close_color = alt.condition("datum.adj_close>datum.adj_open", alt.value("red"), alt.value("green"))
    base = alt.Chart(df).encode(
        alt.X("dt:T").axis(format="%Y-%m-%d", labelAngle=-45),
        color=open_close_color,
        tooltip=["dt", "adj_open", "adj_high", "adj_low", "adj_close"],
    )
    rule = base.mark_rule().encode(alt.Y("adj_low").title("Price"), alt.Y2("adj_high"))
    bar = base.mark_bar(
        fillOpacity=0,  # Make the bar hollow
        strokeWidth=1.5,  # Define the stroke width
    ).encode(
        y="adj_open",
        y2="adj_close",
        stroke=open_close_color,
    )
    return rule + bar


def backtest_chart(uri: str, code: int, start: dt.date, end: dt.date, strategy, chart_width: int = 1600):
    replayer = DuckdbReplayer(start, end, code, uri)
    engine = BacktestEngine(replayer, strategy)
    engine.run()

    chart_ls = calc_ls_chart(strategy.broker.positions)
    chart_candle = calc_candle_chart(uri, code, start, end)
    return (chart_candle + chart_ls).properties(width=1200).configure_scale(zero=False, continuousPadding=50).interactive()
