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
            [row[1:] for row in opened_list],
            symbol="arrow",
            color="blue",
            label_opts=opts.LabelOpts(
                is_show=True,
                position="bottom",
                color="auto",
                formatter=JsCode("""
                    function (params) {
                        return params.value[2];
                    }
                    """),
            ),
        )
        .set_global_opts(
            tooltip_opts=opts.TooltipOpts(
                formatter=JsCode("""
                    function (params) {
                        return 'entry_dt: ' + params.value[0] + '<br/>' +
                            'entry_price: ' + params.value[1] + '<br/>' +
                            'id: ' + params.value[2] + '<br/>' +
                            'volume: ' + params.value[3] + '<br/>' +
                            'pnl: ' + params.value[4];
                    }
                    """)
            ),
            yaxis_opts=opts.AxisOpts(is_scale=True),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )

    closed_list = [
        (
            dt.date(1970, 1, 1) + dt.timedelta(days=pos.exit_dt),
            round(pos.exit_price, 3),
            pos.id,
            pos.volume,
            round(pos.pnl, 3),
            round(pos.fees, 3),
        )
        for pos in positions
        if pos.exit_dt is not None
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
            label_opts=opts.LabelOpts(is_show=False),
        )
        .set_global_opts(
            tooltip_opts=opts.TooltipOpts(
                formatter=JsCode("""
                    function (params) {
                        return 'exit_dt: ' + params.value[0] + '<br/>' +
                            'exit_price: ' + params.value[1] + '<br/>' +
                            'id: ' + params.value[2] + '<br/>' +
                            'volume: ' + params.value[3] + '<br/>' +
                            'pnl: ' + params.value[4] + '<br/>' +
                            'fees: ' + params.value[5];
                    }
                    """)
            ),
            yaxis_opts=opts.AxisOpts(is_scale=True),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )

    return (entry_markers, exit_markers)


def draw_candles(chart_data):
    kline_data = [data[1:-1] for data in chart_data["values"]]
    kline = (
        Kline()
        .add_xaxis(xaxis_data=chart_data["categoryData"])
        .add_yaxis(
            series_name="510050",
            y_axis=kline_data,
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
                    xaxis_index=[0, 1],
                    range_start=95,
                    range_end=100,
                ),
                opts.DataZoomOpts(
                    type_="slider",
                    xaxis_index=[0, 1],
                    range_start=95,
                    range_end=100,
                ),
            ],
            visualmap_opts=opts.VisualMapOpts(
                is_show=False,
                series_index=1,
                is_piecewise=True,
                pieces=[
                    {"value": 1, "color": "#ef232a"},
                    {"value": -1, "color": "#14b143"},
                ],
            ),
            tooltip_opts=opts.TooltipOpts(trigger="axis", axis_pointer_type="cross"),
            axispointer_opts=opts.AxisPointerOpts(
                is_show=True,
                link=[{"xAxisIndex": "all"}],
            ),
        )
    )

    bar = (
        Bar()
        .add_xaxis(xaxis_data=chart_data["categoryData"])
        .add_yaxis(
            series_name="Volume",
            y_axis=chart_data["volumes"],
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

    # kline + Bar
    grid_chart = Grid(init_opts=opts.InitOpts(width="100%", height="700px", theme="vintage"))
    grid_chart.add(kline, grid_opts=opts.GridOpts(pos_left=50, pos_top=20, pos_right=20, height="70%"))
    grid_chart.add(bar, grid_opts=opts.GridOpts(pos_left=50, pos_top="75%", pos_right=20, height="15%"))

    return grid_chart
