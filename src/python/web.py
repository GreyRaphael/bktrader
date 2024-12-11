import os
import datetime as dt

from dotenv import load_dotenv
import secrets
from typing import Annotated
from fastapi import Depends, FastAPI, HTTPException, status, Request
from fastapi.security import HTTPBasic, HTTPBasicCredentials
from fastapi.templating import Jinja2Templates

from bktrader import strategy
from view import backtest_history

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


@app.get("/history/{code}")
async def history_backtest(request: Request, code: int, username: Annotated[str, Depends(get_current_username)]):
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
    return templates.TemplateResponse(request=request, name="index.html", context={"chart_json": chart, "usrname": username})
