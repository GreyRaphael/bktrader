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
    long_scatter = (
        Scatter()
        .add_xaxis([row[0] for row in opened_list])
        .add_yaxis(
            "long",
            [row[1:] for row in opened_list],
            symbol="arrow",
            color="blue",
            label_opts=opts.LabelOpts(is_show=False),
        )
        .set_global_opts(
            tooltip_opts=opts.TooltipOpts(
                formatter=JsCode("""
                    function (params) {
                        return 'entry_dt: ' + params.value[0] + '<br/>' +
                            'entry_price: ' + params.value[1].toFixed(3) + '<br/>' +
                            'id: ' + params.value[2] + '<br/>' +
                            'volume: ' + params.value[3] + '<br/>' +
                            'pnl: ' + params.value[4].toFixed(3);
                    }
                    """)
            ),
            yaxis_opts=opts.AxisOpts(is_scale=True),
            legend_opts=opts.LegendOpts(is_show=False),
        )
    )
