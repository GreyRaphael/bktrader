import os
import datetime as dt
import json

from dotenv import load_dotenv
import secrets
from typing import Annotated
from fastapi import Depends, FastAPI, HTTPException, status, Request
from fastapi.security import HTTPBasic, HTTPBasicCredentials
from fastapi.templating import Jinja2Templates
import duckdb

from bktrader import strategy
from view import backtest_history, backtest_realtime
from draw import backtest_history, backtest_realtime
from quote.realtime import XueQiuQuote
from quote.history import DuckdbReplayer
from engine import BacktestEngine, TradeEngine

# Load environment variables from the .env file (if present)
load_dotenv()

app = FastAPI()
security = HTTPBasic()
templates = Jinja2Templates(directory="templates")


def get_current_username(credentials: Annotated[HTTPBasicCredentials, Depends(security)]):
    current_username_bytes = credentials.username.encode("utf8")
    # Retrieve credentials from environment variables
    correct_username_bytes = os.getenv("BASIC_AUTH_USERNAME").encode("utf8")
    is_correct_username = secrets.compare_digest(current_username_bytes, correct_username_bytes)
    current_password_bytes = credentials.password.encode("utf8")
    # Retrieve credentials from environment variables
    correct_password_bytes = os.getenv("BASIC_AUTH_PASSWORD").encode("utf8")
    is_correct_password = secrets.compare_digest(current_password_bytes, correct_password_bytes)
    if not (is_correct_username and is_correct_password):
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Incorrect username or password", headers={"WWW-Authenticate": "Basic"})
    return credentials.username


@app.get("/history")
async def render_history(
    request: Request,
    username: Annotated[str, Depends(get_current_username)],
):
    return templates.TemplateResponse(request=request, name="history/single.html")


@app.get("/hisback")
async def history_backtest(
    request: Request,
    code: int,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(month=1, day=1),
    end: dt.date = dt.date.today(),
):
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
    uri = "bar1d.db"
    quoter = XueQiuQuote(uri)
    quoter.get_quote(code)
    chart = backtest_history(code, start, end, stg, uri, title=f'{code} {quoter.quote["name"]}')
    # dumpy json is a bad idea, format not match
    return chart.render_embed()


@app.get("/benchmark/history/")
async def bench_history(
    request: Request,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(month=1, day=1),
    end: dt.date = dt.date.today(),
):
    uri = "bar1d.db"
    with duckdb.connect(uri, read_only=True) as conn:
        query = """
        SELECT DISTINCT code 
        FROM etf 
        WHERE
            (
            sector=918 
            OR sector=1000056319000000
            OR sector=1000056320000000
            OR sector=1000056321000000
            OR sector=1000056322000000
            )
            AND dt BETWEEN ? AND ?
        """
        code_list = [code[0] for code in conn.execute(query, [start, end]).fetchall()]

    data = []
    for code in code_list:
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
        replayer = DuckdbReplayer(start, end, code, uri)
        engine = BacktestEngine(replayer, stg)
        engine.run()

        (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
        (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
        row = [
            code,
            round(stg.broker.profit_net(), 3),
            # round(stg.broker.profit_position(), 3),
            round(stg.broker.analyzer.max_drawdown(), 3),
            round(sharpe_annual, 3),
            round(sharpe_volatility, 3),
            round(sharpe_ratio, 3),
            round(sortino_annual, 3),
            round(sortino_volatility, 3),
            round(sortino_ratio, 3),
        ]
        data.append(row)

    bench_json = json.dumps(data)  # can handle nan automatically
    return templates.TemplateResponse(request=request, name="bench.html", context={"usrname": username, "bench_json": bench_json})


@app.get("/realtime/{code}")
async def realtime_backtest(
    request: Request,
    code: int,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(month=1, day=1),
):
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
    uri = "bar1d.db"
    quoter = XueQiuQuote(uri)
    last_quote = quoter.get_quote(code)
    print(last_quote)
    chart = backtest_realtime(code, start, last_quote, stg, uri).to_json()
    return templates.TemplateResponse(request=request, name="index.html", context={"chart_json": chart, "usrname": username})
