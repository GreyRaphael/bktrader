import datetime as dt
from collections import defaultdict
from pyecharts import options as opts
from pyecharts.charts import Kline, Bar, Grid, Scatter
from pyecharts.commons.utils import JsCode


def draw_ls_chart(positions: list):
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
    entry_markers = (
        # Scatter(init_opts=opts.InitOpts(theme="vintage"))
        Scatter()
        .add_xaxis([row[0] for row in opened_list])
        .add_yaxis(
            "entry",
            [row[1:] for row in opened_list],  # multi-dimension data
            symbol="arrow",
            symbol_size=50,
            symbol_rotate=180,
            color="blue",
            label_opts=opts.LabelOpts(
                is_show=True,
                position="bottom",
                color="auto",
                formatter="{@[2]}",  # id, 3rd value; formatter can also be JsCode(visit commit history)
                distance=50,
            ),
        )
        .set_global_opts(
            tooltip_opts=opts.TooltipOpts(
                trigger="item",
                formatter=JsCode("""
                    function (args) {
                        return `exit_dt: ${args.value[0]}<br/>exit_price: ${args.value[1]}<br/>id: ${args.value[2]}<br/>vol: ${args.value[3]}<br/>pnl: ${args.value[4]}`;
                    }
                """),
            ),
            yaxis_opts=opts.AxisOpts(is_scale=True),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )

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

    markpoints = [
        {
            "name": "exit_dt: {}<br/>exit_price: {}<br/>id: {}<br/>vol: {}<br/>pnl: {}<br/>fees:{}".format(*item),  # for js render
            "coord": (item[0], item[1]),
            "symbolSize": 0,  # necessary, make symbol invisible
            "label": {
                "show": True,
                "position": "top",
                "formatter": str(item[2]),
                "color": "brown",
                "fontSize": 12,
                "distance": (item[6] + 1) * 25,  # solve text overlapping
            },
        }
        for item in closed_list
    ]

    exit_markers = (
        # Scatter(init_opts=opts.InitOpts(theme="vintage"))
        Scatter()
        .add_xaxis([row[0] for row in closed_list])
        .add_yaxis(
            "exit",
            [row[1:] for row in closed_list],
            symbol="arrow",
            symbol_rotate=180,
            color="brown",
            label_opts=opts.LabelOpts(is_show=False),  # turn off global markpoint text
            markpoint_opts=opts.MarkPointOpts(
                data=markpoints,
                label_opts=opts.LabelOpts(is_show=False),  # turn off global markpoint text
            ),
        )
        .set_global_opts(
            tooltip_opts=opts.TooltipOpts(
                trigger="item",
                formatter=JsCode("""
                    function (params) {
                        if (params.componentType === 'markPoint') {
                            return params.name;
                        } else {
                            return `exit_dt: ${params.value[0]}<br/>exit_price: ${params.value[1]}`;
                        }
                    }
                """),
            ),
            yaxis_opts=opts.AxisOpts(is_scale=True),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )

    return (entry_markers, exit_markers)


def draw_ls_markers(positions: list):
    up_arrow_svg = "path://M0,0 L10,-10 L20,0 L10,-2 Z"
    down_arrow_svg = "path://M0,0 L10,10 L20,0 L10,2 Z"

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
            "coord": (item[0], item[1]),
            "symbol": up_arrow_svg,
            "symbolSize": 10,
            "itemStyle": {"color": "blue"},
            "label": {
                "show": True,
                "position": "bottom",
                "formatter": str(item[2]),
                "color": "auto",
                "distance": 20,
            },
        }
        for item in opened_list
    ]
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
            "coord": (item[0], item[1]),
            "symbolSize": 10,
            "symbol": down_arrow_svg,
            "itemStyle": {"color": "brown"},
            "label": {
                "show": True,
                "position": "top",
                "formatter": str(item[2]),
                "color": "auto",
                "fontSize": 12,
                "distance": (item[6] + 1) * 20,  # solve text overlapping
            },
        }
        for item in closed_list
    ]

    merged_list = opened_list + closed_list
    markers = (
        # Scatter()
        Scatter(init_opts=opts.InitOpts(theme="vintage"))
        .add_xaxis([row[0] for row in merged_list])
        .add_yaxis(
            "markers",
            y_axis=[row[1] for row in merged_list],
            symbol_size=0,
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


def draw_candles(symbol: str, quotes: list[tuple], positions: list):
    """tuple fields: date,open,close,low,high,volume"""
    # preprocess quotes
    dates = [row[0] for row in quotes]
    oclh = [row[1:-1] for row in quotes]
    vols = [(i, row[-1], -1 if row[1] > row[2] else 1) for i, row in enumerate(quotes)]

    kline = (
        Kline()
        .add_xaxis(xaxis_data=dates)
        .add_yaxis(
            symbol,
            y_axis=oclh,
            itemstyle_opts=opts.ItemStyleOpts(
                color="#ef232a",
                color0="#14b143",
                border_color="#8A0000",
                border_color0="#008F28",
            ),
        )
        .set_global_opts(
            legend_opts=opts.LegendOpts(is_show=True),
            datazoom_opts=[
                opts.DataZoomOpts(
                    type_="inside",
                    xaxis_index=[0, 1],  # kline and volume bars
                    range_start=50,
                    range_end=100,
                ),
                opts.DataZoomOpts(
                    type_="slider",
                    xaxis_index=[0, 1],  # kline and volume bars
                    range_start=50,
                    range_end=100,
                ),
            ],
            visualmap_opts=opts.VisualMapOpts(
                is_show=False,
                series_index=1,  # map to volume bars
                is_piecewise=True,
                pieces=[
                    {"value": 1, "color": "#ef232a"},
                    {"value": -1, "color": "#14b143"},
                ],
            ),
            tooltip_opts=opts.TooltipOpts(
                trigger="item",
                axis_pointer_type="cross",
                formatter=JsCode("""
                    function (params) {
                        if (params.componentType === 'markPoint') {
                            return params.name;
                        } else {
                            return params.seriesName + ': ' + params.value.length + '<br>' + params.value;
                        }
                    }
                """),
            ),
            axispointer_opts=opts.AxisPointerOpts(
                is_show=True,
                # link=[{"xAxisIndex": "all"}],
                link=[{"xAxisIndex": [0, 1]}],
            ),
        )
    )

    bar = (
        Bar()
        .add_xaxis(xaxis_data=dates)
        .add_yaxis(
            series_name="Volume",
            y_axis=vols,
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

    entry_markers, exit_markers = draw_ls_chart(positions)
    overlapped_kline = kline.overlap(entry_markers).overlap(exit_markers)

    # kline + Bar
    grid_chart = Grid(init_opts=opts.InitOpts(width="100%", height="700px", theme="vintage"))
    grid_chart.add(overlapped_kline, grid_opts=opts.GridOpts(pos_left=50, pos_top=20, pos_right=20, height="70%"))
    grid_chart.add(bar, grid_opts=opts.GridOpts(pos_left=50, pos_top="75%", pos_right=20, height="15%"))

    return grid_chart
