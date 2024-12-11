import altair as alt
import datetime as dt
import polars as pl
import duckdb
from engine import BacktestEngine, TradeEngine


# long-short markers and texts
def draw_ls_chart(positions: list):
    if not positions:  # empty positions
        return alt.layer()

    opened_list = [(pos.entry_dt, pos.id, pos.entry_price, pos.volume, pos.pnl) for pos in positions]
    df_opened = pl.from_records(opened_list, orient="row", schema=["dt", "id", "price", "volume", "pnl"]).with_columns(pl.from_epoch("dt", time_unit="d"))
    long_base = alt.Chart(df_opened).encode(alt.X("dt:T").axis(format="%Y-%m-%d", labelAngle=-45), tooltip=["dt", "id", "price", "volume", "pnl"])
    long_markers = long_base.mark_point(shape="triangle-up", color="blue", yOffset=20).encode(y="price")
    long_texts = long_base.mark_text(align="center", baseline="top", yOffset=35, color="blue").encode(y="price", text="id")
    layers = long_markers + long_texts

    closed_list = [(pos.exit_dt, pos.id, pos.exit_price, pos.volume, pos.pnl, pos.fees) for pos in positions if pos.exit_dt is not None]
    if len(closed_list) > 0:
        df_closed = pl.from_records(closed_list, orient="row", schema=["dt", "id", "price", "volume", "pnl", "fees"]).with_columns(pl.from_epoch("dt", time_unit="d"))
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
        layers += short_markers + short_texts
    return layers


def draw_candle_chart(df: pl.DataFrame):
    # Add a vertical rule when hover
    hover = alt.selection_point(fields=["dt"], nearest=True, on="mouseover")
    vertical_rule = alt.Chart(df).mark_rule(color="gray", strokeDash=[5, 5]).encode(x="dt").transform_filter(hover)

    open_close_color = alt.condition("datum.adj_close>datum.adj_open", alt.value("red"), alt.value("green"))
    base = (
        alt.Chart(df)
        .encode(
            alt.X("dt:T", scale=alt.Scale(padding=25)).axis(format="%Y-%m-%d", labelAngle=-45),
            color=open_close_color,
            tooltip=["dt", "adj_open", "adj_high", "adj_low", "adj_close"],
        )
        .add_params(hover)
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
    return rule + bar + vertical_rule


def draw_history_candles(code: int, start: dt.date, end: dt.date, uri: str = "bar1d.db"):
    query = """
    SELECT
        dt,
        ROUND(open * adjfactor / 1e4, 3) AS adj_open,
        ROUND(high * adjfactor / 1e4, 3) AS adj_high,
        ROUND(low * adjfactor / 1e4, 3) AS adj_low,
        ROUND(close * adjfactor / 1e4, 3) AS adj_close,
        volume,
    FROM
        etf
    WHERE 
        code = ? AND dt BETWEEN ? AND ?
    """
    with duckdb.connect(uri) as conn:
        df = conn.execute(query, [code, start, end]).pl()
    return draw_candle_chart(df)


def draw_realtime_candles(code: int, start: dt.date, last_quote, uri: str = "bar1d.db"):
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
        df_history = conn.execute(query, [code, start, dt.date.today()]).pl()

    df_today = pl.DataFrame(
        {
            "dt": last_quote.dt,
            "adj_open": last_quote.open,
            "adj_high": last_quote.high,
            "adj_low": last_quote.low,
            "adj_close": last_quote.close,
        }
    ).with_columns(pl.from_epoch("dt", time_unit="d"))
    df_combined = pl.concat([df_history, df_today])
    return draw_candle_chart(df_combined)


def backtest_history(code: int, start: dt.date, end: dt.date, strategy, uri: str = "bar1d.db"):
    from quote.history import DuckdbReplayer

    replayer = DuckdbReplayer(start, end, code, uri)
    engine = BacktestEngine(replayer, strategy)
    engine.run()

    chart_ls = draw_ls_chart(strategy.broker.positions)
    chart_candle = draw_history_candles(code, start, end, uri)
    return (chart_candle + chart_ls).properties(width="container", height=700, title=str(code)).configure_scale(zero=False).interactive()


def backtest_realtime(code: int, start: dt.date, last_quote, strategy, uri: str = "bar1d.db"):
    from quote.history import DuckdbReplayer

    end = dt.date.today()
    replayer = DuckdbReplayer(start, end, code, uri)
    engine = TradeEngine(replayer, last_quote, strategy)
    engine.run()

    chart_ls = draw_ls_chart(strategy.broker.positions)
    chart_candle = draw_realtime_candles(code, start, last_quote, uri)
    return (chart_candle + chart_ls).properties(width="container", height=700, title=str(code)).configure_scale(zero=False).interactive()


def benchmark_strategy(stg):
    print(f"profit_net: {stg.broker.profit_net():.3f}, profit_gross:{stg.broker.profit_gross():.3f}")
    print(f"max_drawdown: {stg.broker.analyzer.max_drawdown():.3f}")

    annual_return, annual_volatility, sharpe_ratio = stg.broker.analyzer.sharpe_ratio(0.015)
    print(f"sharpe annual_return: {annual_return:.3f}")
    print(f"sharpe annual_volatility: {annual_volatility:.3f}")
    print(f"sharpe sharpe_ratio: {sharpe_ratio:.3f}")

    annual_return, annual_downside_deviation, sortino_ratio = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
    print(f"sortino annual_return: {annual_return:.3f}")
    print(f"sortino annual_downside_deviation: {annual_downside_deviation:.3f}")
    print(f"sortino sortino_ratio: {sortino_ratio:.3f}")


def history(args):
    from bktrader import strategy

    alt.renderers.enable("browser")
    stg = strategy.GridCCI(
        init_cash=1e5,
        cum_quantile=0.3,
        rank_period=15,
        rank_limit=0.3,
        cci_threshold=0.0,
        max_active_pos_len=25,
        profit_limit=0.15,
        # profit_limit=0.08,
    )

    chart = backtest_history(args.code, args.start_dt, args.end_dt, stg, args.uri)
    benchmark_strategy(stg)
    chart.show()


def realtime(args):
    from bktrader import strategy
    from quote.realtime import XueQiuQuote

    alt.renderers.enable("browser")
    stg = strategy.GridCCI(
        init_cash=2e5,
        cum_quantile=0.3,
        rank_period=15,
        rank_limit=0.3,
        cci_threshold=0.0,
        max_active_pos_len=50,
        profit_limit=0.15,
        # profit_limit=0.08,
    )

    quoter = XueQiuQuote(args.uri)
    last_quote = quoter.get_quote(args.code)
    chart = backtest_realtime(args.code, args.start_dt, last_quote, stg, args.uri)
    benchmark_strategy(stg)
    chart.show()


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="backtest 1 etf in history or realtime")
    today_date = dt.date.today()
    parser.add_argument("-sdt", dest="start_dt", type=lambda s: dt.datetime.strptime(s, "%Y%m%d").date(), default=dt.date(today_date.year, 1, 1), help="start date")
    parser.add_argument("-c", dest="code", type=int, required=True, help="etf integer code")
    parser.add_argument("-uri", type=str, default="bar1d.db", help="duckdb uri")

    subparsers = parser.add_subparsers(description='choose from ["history", "realtime"]', required=True)

    backtester = subparsers.add_parser("history", help="backtest 1 etf in history")
    backtester.set_defaults(func=history)
    backtester.add_argument("-edt", dest="end_dt", type=lambda s: dt.datetime.strptime(s, "%Y%m%d").date(), default=today_date, help="end date")

    trader = subparsers.add_parser("realtime", help="backtest 1 etf in realtime")
    trader.set_defaults(func=realtime)

    args = parser.parse_args()
    args.func(args)
