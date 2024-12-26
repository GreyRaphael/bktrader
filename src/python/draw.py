import datetime as dt
from collections import defaultdict
from pyecharts import options as opts
from pyecharts.charts import Kline, Bar, Grid, Scatter
from pyecharts.commons.utils import JsCode
import duckdb


# long-short markers and texts
def draw_trade_markers(positions: list):
    up_arrow_svg = "path://M0,0 L10,-10 L20,0 L10,-2 Z"
    down_arrow_svg = "path://M0,0 L10,10 L20,0 L10,2 Z"

    # prepare entry markpoints
    opened_list = [
        (
            dt.date(1970, 1, 1) + dt.timedelta(days=pos.entry_dt),
            round(pos.entry_price, 3),
            pos.id,
            pos.volume,
            round(pos.pnl, 3),
        )
        for pos in positions
    ]
    entry_markpoints = [
        {
            "name": "entry_dt: {}<br/>entry_price: {}<br/>id: {}<br/>vol: {}<br/>pnl: {}".format(*item),  # for js render
            "coord": (item[0], item[1] * 0.98),
            "symbol": up_arrow_svg,
            "symbolSize": 10,
            "itemStyle": {"color": "blue"},
            "label": {
                "show": True,
                "position": "bottom",
                "color": "auto",
                "distance": 13,
                "fontSize": 10,
                "formatter": str(item[2]),
            },
        }
        for item in opened_list
    ]

    # prepare exit markpoints
    counts = defaultdict(int)
    closed_list = []
    # groupby (dt, price) to solve text overlapping
    for pos in positions:
        if pos.exit_dt is not None:
            x = dt.date(1970, 1, 1) + dt.timedelta(days=pos.exit_dt)
            y = round(pos.exit_price, 3)
            closed_list.append(
                (
                    x,
                    y,
                    pos.id,
                    pos.volume,
                    round(pos.pnl, 3),
                    round(pos.fees, 3),
                    counts[(x, y)],
                )
            )
            counts[(x, y)] += 1
    exit_markpoints = [
        {
            "name": "exit_dt: {}<br/>exit_price: {}<br/>id: {}<br/>vol: {}<br/>pnl: {}<br/>fees:{}".format(*item),  # for js render
            "coord": (item[0], item[1] * 1.02),
            "symbol": down_arrow_svg,
            "symbolSize": 10,
            "itemStyle": {"color": "magenta"},
            "label": {
                "show": True,
                "position": "top",
                "color": "auto",
                "distance": item[6] * 13 + 6,  # solve text overlapping
                "fontSize": 10,
                "formatter": str(item[2]),
            },
        }
        for item in closed_list
    ]

    # merge opened and closed
    merged_list = opened_list + closed_list
    markers = (
        Scatter()
        # Scatter(init_opts=opts.InitOpts(theme="vintage")) # for debug
        .add_xaxis([row[0] for row in merged_list])
        .add_yaxis(
            "markers",
            y_axis=[row[1] for row in merged_list],
            symbol_size=0,  # necessary, make origin symbol invisible
            label_opts=opts.LabelOpts(is_show=False),
            markpoint_opts=opts.MarkPointOpts(
                data=entry_markpoints + exit_markpoints,
                label_opts=opts.LabelOpts(is_show=False),  # turn off global markpoint text
            ),
        )
        .set_global_opts(
            tooltip_opts=opts.TooltipOpts(formatter=JsCode("function (params) {return params.name;}")),
            yaxis_opts=opts.AxisOpts(is_scale=True),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )
    return markers


def draw_candles_with_markers(quotes: list[tuple], positions: list, title: str = None):
    """tuple fields: date,open,close,low,high,volume"""
    # preprocess quotes
    dates = [row[0] for row in quotes]
    oclhp = [row[1:-1] for row in quotes]
    vol_bar_items = [
        {
            "value": row[-1],
            "itemStyle": {"color": "green" if row[1] >= row[2] else "red"},
        }
        for row in quotes
    ]

    # prepare volume bars
    vol_bars = (
        Bar()
        .add_xaxis(xaxis_data=dates)
        .add_yaxis(
            series_name="Volumes",
            y_axis=vol_bar_items,
            bar_width="80%",
            label_opts=opts.LabelOpts(is_show=False),
        )
        .set_global_opts(
            xaxis_opts=opts.AxisOpts(
                axistick_opts=opts.AxisTickOpts(is_show=False),
                axislabel_opts=opts.LabelOpts(is_show=False),
                splitline_opts=opts.SplitLineOpts(is_show=False),
            ),
            yaxis_opts=opts.AxisOpts(
                is_scale=True,
                axislabel_opts=opts.LabelOpts(is_show=False),
                splitline_opts=opts.SplitLineOpts(is_show=False),
            ),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )

    # prepare kline
    kline = (
        Kline()
        .add_xaxis(xaxis_data=dates)
        .add_yaxis(
            "Candles",
            y_axis=oclhp,
            bar_width="80%",
            itemstyle_opts=opts.ItemStyleOpts(
                color="red",
                color0="green",
                border_color="#8A0000",
                border_color0="#008F28",
            ),
        )
        .set_global_opts(
            title_opts=opts.TitleOpts(title=title, pos_left="center"),
            legend_opts=opts.LegendOpts(is_show=False),
            yaxis_opts=opts.AxisOpts(is_scale=True, boundary_gap=["0%", "20%"]),
            datazoom_opts=[
                opts.DataZoomOpts(
                    type_="inside",
                    xaxis_index=[0, 1],  # kline and volume bars
                    range_start=0,
                    range_end=100,
                ),
                opts.DataZoomOpts(
                    type_="slider",
                    xaxis_index=[0, 1],  # kline and volume bars
                    range_start=0,
                    range_end=100,
                ),
            ],
            tooltip_opts=opts.TooltipOpts(
                trigger="item",
                axis_pointer_type="cross",
                formatter=JsCode("""
                    function (args) {
                        if (args.componentType === 'markPoint') {
                            return args.name;
                        } else if (args.seriesName === 'Volumes') {
                            return `${args.seriesName}: ${args.value}`;
                        } else if (args.seriesName === 'Candles') {
                            var pct = args.value[5];
                            var pctColor = pct > 0 ? 'red' : 'green';
                            return `date: ${args.name}<br/>open: ${args.value[1]}<br/>close: ${args.value[2]}<br/>low: ${args.value[3]}<br/>high: ${args.value[4]}<br/>pct: <span style="color: ${pctColor};">${pct}%</span>`;
                        }
                    }
                """),
            ),
            axispointer_opts=opts.AxisPointerOpts(
                is_show=True,
                link=[{"xAxisIndex": "all"}],  # link all axis
            ),
        )
    )

    markers = draw_trade_markers(positions)
    kline_with_markers = kline.overlap(markers)

    # kline + Bar
    # grid_chart = Grid(init_opts=opts.InitOpts(width="100%", height="700px", theme="vintage"))  # for jupyter debug
    grid_chart = Grid(init_opts=opts.InitOpts(width="100%", height="700px"))
    grid_chart.add(kline_with_markers, grid_opts=opts.GridOpts(pos_left="1%", pos_top="5%", pos_right="0.5%", height="70%", is_contain_label=True))  # is_contain_label 始终让label在里面
    grid_chart.add(vol_bars, grid_opts=opts.GridOpts(pos_left="1%", pos_top="75%", pos_right="0.5%", height="15%"))

    return grid_chart


def fetch_history_candles(code: int, start: dt.date, end: dt.date, uri: str) -> list[tuple]:
    query = """
    SELECT
        dt,
        ROUND(open * adjfactor / 1e4, 3) AS adj_open,
        ROUND(close * adjfactor / 1e4, 3) AS adj_close,
        ROUND(low * adjfactor / 1e4, 3) AS adj_low,
        ROUND(high * adjfactor / 1e4, 3) AS adj_high,
        ROUND(100 * (close / preclose - 1), 2) AS pct,
        volume,
    FROM
        bar1d
    WHERE 
        code = ? AND dt BETWEEN ? AND ?
    """
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute(query, [code, start, end]).fetchall()
    return records


def fetch_realtime_candles(code: int, start: dt.date, last_quote, uri: str) -> list[tuple]:
    query = """
    SELECT
        dt,
        ROUND(open * adjfactor / 1e4, 3) AS adj_open,
        ROUND(close * adjfactor / 1e4, 3) AS adj_close,
        ROUND(low * adjfactor / 1e4, 3) AS adj_low,
        ROUND(high * adjfactor / 1e4, 3) AS adj_high,
        ROUND(100 * (close / preclose - 1), 2) AS pct,
        volume,
    FROM
        bar1d
    WHERE 
        code = ? AND dt BETWEEN ? AND ?
    """
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute(query, [code, start, dt.date.today()]).fetchall()
    record_today = (
        dt.date(1970, 1, 1) + dt.timedelta(days=last_quote.dt),
        last_quote.open,
        last_quote.close,
        last_quote.low,
        last_quote.high,
        round(100 * (last_quote.close / last_quote.preclose - 1), 2),
        last_quote.volume,
    )
    return records + [record_today]


def backtest_history(code: int, start: dt.date, end: dt.date, strategy, uri: str, title: str = None):
    from quote.history import DuckdbReplayer
    from engine import BacktestEngine

    replayer = DuckdbReplayer(start, end, code, uri)
    engine = BacktestEngine(replayer, strategy)
    engine.run()

    quotes = fetch_history_candles(code, start, end, uri)
    chart = draw_candles_with_markers(quotes, strategy.broker.positions, title)
    return chart


def backtest_realtime(code: int, start: dt.date, last_quote, strategy, uri: str, title: str = None):
    from quote.history import DuckdbReplayer
    from engine import TradeEngine

    end = dt.date.today()
    replayer = DuckdbReplayer(start, end, code, uri)
    engine = TradeEngine(replayer, last_quote, strategy)
    engine.run()

    quotes = fetch_realtime_candles(code, start, last_quote, uri)
    chart = draw_candles_with_markers(quotes, strategy.broker.positions, title)
    return chart
