import os
import datetime as dt
import json

from dotenv import load_dotenv
import secrets
from typing import Annotated
from fastapi import Depends, FastAPI, HTTPException, status, Request
from fastapi.security import HTTPBasic, HTTPBasicCredentials
from fastapi.templating import Jinja2Templates
from fastapi.staticfiles import StaticFiles

from bktrader import strategy
from draw import backtest_history, backtest_realtime
from quote.realtime import XueQiuQuote, EastEtfQuote, EastLofQuote
from quote.history import DuckBatchReplayer
from quote.fundtype import ETFType, LOFType
from quote import info

# Load environment variables from the .env file (if present)
load_dotenv()
ETF_DB_URI = os.getenv("ETF_DB_URI")
LOF_DB_URI = os.getenv("LOF_DB_URI")
ETF_INFO_DICT = info.query_info_all(ETF_DB_URI)  # {code: (name, mer, cer)}
LOF_INFO_DICT = info.query_info_all(LOF_DB_URI)

app = FastAPI()
security = HTTPBasic()
templates = Jinja2Templates(directory="templates")

# add static js or css
app.mount("/static", StaticFiles(directory="static"), name="static")


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


@app.get("/etf/history/{code}")
async def render_etf_history(
    request: Request,
    code: int,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    end: dt.date = dt.date.today(),
    profit: int = 15,
):
    stg = strategy.GridCCI(
        init_cash=1e5,
        cum_quantile=0.3,
        rank_period=15,
        rank_limit=0.3,
        cci_threshold=0.0,
        max_active_pos_len=25,
        profit_limit=profit / 1e2,
    )
    name, mer, cer = ETF_INFO_DICT.get(code, (None, None, None))
    chart = backtest_history(code, start, end, stg, ETF_DB_URI, title=f"{code} {name}")

    (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
    (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
    avg_hold_days = round(stg.broker.avg_hold_days(), 3)

    return templates.TemplateResponse(
        request=request,
        name="history/single.html",
        context={
            "code": code,
            "portfolio_profit": round(stg.broker.profit_net(), 3),
            "max_drawdown": round(stg.broker.analyzer.max_drawdown(), 3),
            "sharpe_annual": round(sharpe_annual, 3),
            "sharpe_volatility": round(sharpe_volatility, 3),
            "sharpe_ratio": round(sharpe_ratio, 3),
            "sortino_annual": round(sortino_annual, 3),
            "sortino_volatility": round(sortino_volatility, 3),
            "sortino_ratio": round(sortino_ratio, 3),
            "mer": mer,
            "cer": cer,
            "avg_hold_days": avg_hold_days,
            "candles": chart.render_embed(),
        },
    )


@app.get("/lof/history/{code}")
async def render_lof_history(
    request: Request,
    code: int,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    end: dt.date = dt.date.today(),
    profit: int = 8,
):
    stg = strategy.GridCCI(
        init_cash=1e5,
        cum_quantile=0.3,
        rank_period=15,
        rank_limit=0.3,
        cci_threshold=0.0,
        max_active_pos_len=25,
        profit_limit=profit / 1e2,
    )
    name, mer, cer = LOF_INFO_DICT.get(code, (None, None, None))
    chart = backtest_history(code, start, end, stg, LOF_DB_URI, title=f"{code} {name}")

    (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
    (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
    avg_hold_days = round(stg.broker.avg_hold_days(), 3)

    return templates.TemplateResponse(
        request=request,
        name="history/single.html",
        context={
            "code": code,
            "portfolio_profit": round(stg.broker.profit_net(), 3),
            "max_drawdown": round(stg.broker.analyzer.max_drawdown(), 3),
            "sharpe_annual": round(sharpe_annual, 3),
            "sharpe_volatility": round(sharpe_volatility, 3),
            "sharpe_ratio": round(sharpe_ratio, 3),
            "sortino_annual": round(sortino_annual, 3),
            "sortino_volatility": round(sortino_volatility, 3),
            "sortino_ratio": round(sortino_ratio, 3),
            "mer": mer,
            "cer": cer,
            "avg_hold_days": avg_hold_days,
            "candles": chart.render_embed(),
        },
    )


@app.get("/etf/realtime/{code}")
async def render_etf_realtime(
    request: Request,
    code: int,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    profit: int = 15,
):
    stg = strategy.GridCCI(
        init_cash=1e5,
        cum_quantile=0.3,
        rank_period=15,
        rank_limit=0.3,
        cci_threshold=0.0,
        max_active_pos_len=25,
        profit_limit=profit / 1e2,
    )
    name, mer, cer = ETF_INFO_DICT.get(code, (None, None, None))
    quoter = XueQiuQuote(ETF_DB_URI)
    last_quote = quoter.get_quote(code)
    discount = round((quoter.quote["current"] / quoter.quote["iopv"] - 1) * 100, 3)
    chart = backtest_realtime(code, start, last_quote, stg, ETF_DB_URI, title=f"{code} {name}")

    (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
    (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
    avg_hold_days = round(stg.broker.avg_hold_days(), 3)

    return templates.TemplateResponse(
        request=request,
        name="realtime/single.html",
        context={
            "code": code,
            "portfolio_profit": round(stg.broker.profit_net(), 3),
            "max_drawdown": round(stg.broker.analyzer.max_drawdown(), 3),
            "sharpe_annual": round(sharpe_annual, 3),
            "sharpe_volatility": round(sharpe_volatility, 3),
            "sharpe_ratio": round(sharpe_ratio, 3),
            "sortino_annual": round(sortino_annual, 3),
            "sortino_volatility": round(sortino_volatility, 3),
            "sortino_ratio": round(sortino_ratio, 3),
            "mer": mer,
            "cer": cer,
            "avg_hold_days": avg_hold_days,
            "discount": discount,
            "candles": chart.render_embed(),
        },
    )


@app.get("/lof/realtime/{code}")
async def render_lof_realtime(
    request: Request,
    code: int,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    profit: int = 8,
):
    stg = strategy.GridCCI(
        init_cash=1e5,
        cum_quantile=0.3,
        rank_period=15,
        rank_limit=0.3,
        cci_threshold=0.0,
        max_active_pos_len=25,
        profit_limit=profit / 1e2,
    )
    name, mer, cer = LOF_INFO_DICT.get(code, (None, None, None))
    quoter = XueQiuQuote(LOF_DB_URI)
    last_quote = quoter.get_quote(code)
    chart = backtest_realtime(code, start, last_quote, stg, LOF_DB_URI, title=f"{code} {name}")

    (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
    (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
    avg_hold_days = round(stg.broker.avg_hold_days(), 3)

    return templates.TemplateResponse(
        request=request,
        name="realtime/single.html",
        context={
            "code": code,
            "portfolio_profit": round(stg.broker.profit_net(), 3),
            "max_drawdown": round(stg.broker.analyzer.max_drawdown(), 3),
            "sharpe_annual": round(sharpe_annual, 3),
            "sharpe_volatility": round(sharpe_volatility, 3),
            "sharpe_ratio": round(sharpe_ratio, 3),
            "sortino_annual": round(sortino_annual, 3),
            "sortino_volatility": round(sortino_volatility, 3),
            "sortino_ratio": round(sortino_ratio, 3),
            "mer": mer,
            "cer": cer,
            "avg_hold_days": avg_hold_days,
            "discount": float("nan"),
            "candles": chart.render_embed(),
        },
    )


@app.get("/etf/history/")
async def bench_etf_history(
    request: Request,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    end: dt.date = dt.date.today(),
    profit: int = 15,
    xt: ETFType = "qdii",
):
    if ETFType.commodity == xt:
        sectors = [1000010087000000]
    elif ETFType.bond == xt:
        sectors = [1000009166000000]
    elif ETFType.stock == xt:
        sectors = [1000009712000000, 1000009713000000, 1000009714000000, 1000009715000000, 1000009716000000]
    else:
        # default is qdii
        sectors = [918, 1000056319000000, 1000056320000000, 1000056321000000, 1000056322000000]
    code_list = info.query_sector_codes(ETF_DB_URI, sectors)

    stgs = {
        code: strategy.GridCCI(
            init_cash=1e5,
            cum_quantile=0.3,
            rank_period=15,
            rank_limit=0.3,
            cci_threshold=0.0,
            max_active_pos_len=25,
            profit_limit=profit / 1e2,
        )
        for code in code_list
    }

    replayer = DuckBatchReplayer(start, end, code_list, ETF_DB_URI)
    for quote in replayer:
        stgs[quote.code].on_update(quote)

    data = []
    for code in stgs:
        stg = stgs[code]
        (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
        (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
        name, mer, cer = ETF_INFO_DICT.get(code, (None, None, None))
        row = [
            code,
            name,
            mer,
            cer,
            round(stg.broker.profit_net(), 3),
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
    return templates.TemplateResponse(request=request, name="history/etf_summary.html", context={"bench_json": bench_json})


@app.get("/lof/history/")
async def bench_lof_history(
    request: Request,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    end: dt.date = dt.date.today(),
    profit: int = 8,
    xt: LOFType = "qdii",
):
    if LOFType.commodity == xt:
        sectors = [1000043336000000]
    elif LOFType.bond == xt:
        sectors = [1000043335000000]
    else:
        # default is qdii+commodity
        sectors = [1000043337000000, 1000043336000000]
    code_list = info.query_sector_codes(LOF_DB_URI, sectors)

    stgs = {
        code: strategy.GridCCI(
            init_cash=1e5,
            cum_quantile=0.3,
            rank_period=15,
            rank_limit=0.3,
            cci_threshold=0.0,
            max_active_pos_len=25,
            profit_limit=profit / 1e2,
        )
        for code in code_list
    }

    replayer = DuckBatchReplayer(start, end, code_list, LOF_DB_URI)
    for quote in replayer:
        stgs[quote.code].on_update(quote)

    data = []
    for code in stgs:
        stg = stgs[code]
        (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
        (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
        name, mer, cer = LOF_INFO_DICT.get(code, (None, None, None))
        row = [
            code,
            name,
            mer,
            cer,
            round(stg.broker.profit_net(), 3),
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
    return templates.TemplateResponse(request=request, name="history/lof_summary.html", context={"bench_json": bench_json})


@app.get("/etf/realtime/")
async def etf_available(
    request: Request,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    profit: int = 15,
    xt: ETFType = "qdii",
):
    # download real time quotes
    quoter = EastEtfQuote(ETF_DB_URI, xt)
    quoter.update()
    latest_quotes = quoter.get_quotes()
    code_list = quoter.latest_bars.keys()

    stgs = {
        code: strategy.GridCCI(
            init_cash=1e5,
            cum_quantile=0.3,
            rank_period=15,
            rank_limit=0.3,
            cci_threshold=0.0,
            max_active_pos_len=25,
            profit_limit=profit / 1e2,
        )
        for code in code_list
    }

    replayer = DuckBatchReplayer(start, dt.date.today(), code_list, ETF_DB_URI)
    for quote in replayer:
        stgs[quote.code].on_update(quote)
    for quote in latest_quotes:
        stgs[quote.code].on_update(quote)

    data = []
    for code in stgs:
        stg = stgs[code]

        last_position = stg.broker.position_last()
        if last_position:
            (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
            (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
            name, mer, cer = ETF_INFO_DICT.get(code, (None, None, None))
            row = [
                code,
                name,
                (dt.date(1970, 1, 1) + dt.timedelta(days=last_position.entry_dt)).isoformat() if last_position.entry_dt else None,
                (dt.date(1970, 1, 1) + dt.timedelta(days=last_position.exit_dt)).isoformat() if last_position.exit_dt else None,
                mer,
                cer,
                round(stg.broker.profit_net(), 3),
                round(stg.broker.analyzer.max_drawdown(), 3),
                round(sharpe_annual, 3),
                round(sharpe_volatility, 3),
                round(sharpe_ratio, 3),
                round(sortino_annual, 3),
                round(sortino_volatility, 3),
                round(sortino_ratio, 3),
            ]
            data.append(row)

    available_json = json.dumps(data)  # can handle nan automatically
    return templates.TemplateResponse(request=request, name="realtime/etf_summary.html", context={"available_json": available_json})


@app.get("/lof/realtime/")
async def lof_available(
    request: Request,
    username: Annotated[str, Depends(get_current_username)],
    start: dt.date = dt.date.today().replace(year=dt.date.today().year - 2),
    profit: int = 8,
    xt: LOFType = "qdii",
):
    # download real time quotes
    quoter = EastLofQuote(LOF_DB_URI, xt)
    quoter.update()
    latest_quotes = quoter.get_quotes()
    code_list = quoter.latest_bars.keys()

    stgs = {
        code: strategy.GridCCI(
            init_cash=1e5,
            cum_quantile=0.3,
            rank_period=15,
            rank_limit=0.3,
            cci_threshold=0.0,
            max_active_pos_len=25,
            profit_limit=profit / 1e2,
        )
        for code in code_list
    }

    replayer = DuckBatchReplayer(start, dt.date.today(), code_list, LOF_DB_URI)
    for quote in replayer:
        stgs[quote.code].on_update(quote)
    for quote in latest_quotes:
        stgs[quote.code].on_update(quote)

    data = []
    for code in stgs:
        stg = stgs[code]
        last_position = stg.broker.position_last()
        if last_position:
            (sharpe_annual, sharpe_volatility, sharpe_ratio) = stg.broker.analyzer.sharpe_ratio(0.015)
            (sortino_annual, sortino_volatility, sortino_ratio) = stg.broker.analyzer.sortino_ratio(0.015, 0.01)
            name, mer, cer = LOF_INFO_DICT.get(code, (None, None, None))
            row = [
                code,
                name,
                (dt.date(1970, 1, 1) + dt.timedelta(days=last_position.entry_dt)).isoformat() if last_position.entry_dt else None,
                (dt.date(1970, 1, 1) + dt.timedelta(days=last_position.exit_dt)).isoformat() if last_position.exit_dt else None,
                mer,
                cer,
                round(stg.broker.profit_net(), 3),
                round(stg.broker.analyzer.max_drawdown(), 3),
                round(sharpe_annual, 3),
                round(sharpe_volatility, 3),
                round(sharpe_ratio, 3),
                round(sortino_annual, 3),
                round(sortino_volatility, 3),
                round(sortino_ratio, 3),
            ]
            data.append(row)

    available_json = json.dumps(data)  # can handle nan automatically
    return templates.TemplateResponse(request=request, name="realtime/lof_summary.html", context={"available_json": available_json})
