import datetime as dt
from fastapi import FastAPI, Request
from fastapi.templating import Jinja2Templates
from bktrader import strategy
from view import backtest_history

app = FastAPI()
templates = Jinja2Templates(directory="templates")


@app.get("/history/{code}")
async def read_item(request: Request, code: int):
    dt_today = dt.date.today()
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
    chart = backtest_history(code, dt.date(dt_today.year, 1, 1), dt_today, stg, "bar1d.db", chart_width=1800).to_json()
    return templates.TemplateResponse(request=request, name="index.html", context={"chart_json": chart})
