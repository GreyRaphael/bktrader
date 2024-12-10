import math
import datetime as dt
import httpx
import duckdb
from bktrader import datatype


def predicted_close_ratio(update_time: dt.datetime):
    morning_start = dt.datetime.combine(update_time.date(), dt.time(9, 30, 0))
    morning_end = dt.datetime.combine(update_time.date(), dt.time(11, 30, 0))
    afternoon_start = dt.datetime.combine(update_time.date(), dt.time(13, 0, 0))
    afternoon_end = dt.datetime.combine(update_time.date(), dt.time(15, 0, 0))
    if update_time < morning_start:
        time_ratio = 0.0
    elif update_time <= morning_end:
        time_ratio = (update_time - morning_start) / dt.timedelta(hours=4)
    elif update_time < afternoon_start:
        time_ratio = 0.5
    elif update_time <= afternoon_end:
        time_ratio = (update_time - afternoon_start + dt.timedelta(hours=2)) / dt.timedelta(hours=4)
    else:
        time_ratio = 1.0
    return time_ratio


class EastQuote:
    """
    get quotes of all etf
    source: http://quote.eastmoney.com/center/gridlist.html#fund_etf
    """

    def __init__(self, uri: str):
        self.uri = uri
        self.client = httpx.Client(headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0"})
        self.mapping = {
            "f12": "code",
            "f297": "dt",
            "f18": "preclose",
            "f17": "open",
            "f15": "high",
            "f16": "low",
            "f2": "last",
            "f5": "volume",
            "f6": "amount",
            "f441": "iopv",
            "f124": "update_time",
        }
        self.latest_bars = {}

    def days_since_epoch(self, value: int) -> int:
        year = value // 10000  # 2024
        month = (value % 10000) // 100  # 12
        day = value % 100  # 06
        delta = dt.date(year, month, day) - dt.date(1970, 1, 1)
        return delta.days

    def update(self):
        # MK0021: A
        # MK0022: currency
        # MK0023: foreign
        # MK0024: gold
        # MK0827: commodity
        url_params = {
            "pn": 1,  # page number
            "pz": 2000,  # page size > etf total size
            "po": 1,  # page offset
            "np": 1,
            "ut": "bd1d9ddb04089700cf9c27f6f7426281",
            "fltt": 2,
            "invt": 2,  # inverse
            "dect": 1,
            "wbp2u": "|0|0|0|web",
            "fid": "f5",  # sort by volume
            "fs": "b:MK0023",  # market type: b:MK0021,b:MK0022,b:MK0023,b:MK0024,b:MK0827
            "fields": ",".join(self.mapping.keys()),
            "_": int(dt.datetime.now().timestamp() * 1000),  # timestamp
        }
        rsp = self.client.get("http://push2.eastmoney.com/api/qt/clist/get", params=url_params, timeout=5).json()

        for record in rsp["data"]["diff"]:
            update_time = dt.datetime.fromtimestamp(record["f124"])  # run after 14:00
            time_ratio = predicted_close_ratio(update_time)

            predicted_today_volume = record["f5"] * 1e2 / time_ratio  # 1e2单位手换算成股
            predicted_today_amount = record["f6"] / time_ratio

            code = int(record["f12"])
            self.latest_bars[code] = (
                code,  # code
                self.days_since_epoch(record["f297"]),  # dt
                record["f18"],  # preclose
                record["f17"],  # open
                record["f15"],  # high
                record["f16"],  # low
                record["f2"],  # last
                predicted_today_volume,  # volume
                predicted_today_amount,  # amount
            )

    def get_quote(self, code: int) -> datatype.Bar:
        with duckdb.connect(self.uri, read_only=True) as conn:
            query = "SELECT ROUND(close/1e4, 3), adjfactor FROM etf WHERE code=? ORDER BY dt DESC LIMIT 1"
            duck_last_close, factor = conn.execute(query, [code]).fetchone()
        code, dt, preclose, open, high, low, last, predicted_vol, predicted_amt = self.latest_bars[code]
        adjfactor = factor if math.isclose(duck_last_close, preclose) else duck_last_close / preclose * factor
        # print("prev", factor, "now", adjfactor)
        return datatype.Bar(
            code=code,
            dt=dt,
            preclose=round(preclose * adjfactor, 3),
            open=round(open * adjfactor, 3),
            high=round(high * adjfactor, 3),
            low=round(low * adjfactor, 3),
            close=round(last * adjfactor, 3),
            volume=predicted_vol,
            amount=round(predicted_amt * adjfactor, 3),
        )


class EastSingleQuote:
    """
    get quote of single stock or etf
    not recommended bad than XueQiuQuote
    source: http://quote.eastmoney.com/sz159526.html
    """

    def __init__(self, uri: str):
        self.uri = uri
        self.client = httpx.Client(headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0"})
        self.mapping = {
            "f57": "code",
            "f86": "update_time",
            "f60": "preclose",
            "f46": "open",
            "f44": "high",
            "f45": "low",
            "f43": "last",
            "f47": "volume",  # 手
            "f48": "amount",
            # "f168": "turnover",
            # "f49": "tot_bid",  # buy
            # "f161": "tot_ask",
            # "f50": "qrr",
            # "f51": "high_limit",
            # "f52": "low_limit",
            # "f71": "vwap",
            # 'f84':'tot_share',
            # 'f85':'float_share',
            # 'f116':'tot_mkt_val',
            # 'f117':'float_mkt_val',
        }

    def get_quote(self, code: int):
        url_params = {
            "secid": f"1.{code}" if code > 500000 else f"0.{code}",
            "ut": "f057cbcbce2a86e2866ab8877db1d059",
            "_": int(dt.datetime.now().timestamp() * 1000),
        }
        quote = self.client.get("https://push2.eastmoney.com/api/qt/stock/get", params=url_params, timeout=5).json()["data"]
        return quote


class XueQiuQuote:
    """
    get quote of single code (can also for stock or etf)
    source: https://xueqiu.com/S/SH600000
    """

    def __init__(self, uri: str):
        self.uri = uri
        self.client = httpx.Client(
            headers={
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0",
                "Cookie": "xq_a_token=220b0abef0fac476d076c9f7a3938b7edac35f48;",
            }
        )

    def get_quote(self, code: int) -> datatype.Bar:
        with duckdb.connect(self.uri, read_only=True) as conn:
            query = "SELECT ROUND(close/1e4, 3), adjfactor FROM etf WHERE code=? ORDER BY dt DESC LIMIT 1"
            duck_last_close, factor = conn.execute(query, [code]).fetchone()

        url_params = {
            "symbol": f"SH{code}" if code > 500000 else f"SZ{code}",
            "extend": "detail",
        }
        url = "https://stock.xueqiu.com/v5/stock/quote.json"
        quote = self.client.get(url, params=url_params, timeout=5).json()["data"]["quote"]
        # iopv = quote["iopv"]
        # netvalue = quote["unit_nav"]
        update_dt = dt.datetime.fromtimestamp(quote["time"] / 1000)
        time_ratio = predicted_close_ratio(update_dt)
        preclose = quote["last_close"]
        adjfactor = factor if math.isclose(duck_last_close, preclose) else duck_last_close / preclose * factor
        # print("prev", factor, "now", adjfactor)
        return datatype.Bar(
            code=int(quote["code"]),
            dt=(update_dt.date() - dt.date(1970, 1, 1)).days,
            preclose=round(preclose * adjfactor, 3),
            open=round(quote["open"] * adjfactor, 3),
            high=round(quote["high"] * adjfactor, 3),
            low=round(quote["low"] * adjfactor, 3),
            close=round(quote["current"] * adjfactor, 3),
            volume=quote["volume"] / time_ratio,
            amount=round(quote["amount"] / time_ratio * adjfactor, 3),
        )


if __name__ == "__main__":
    east_quoter = EastQuote(uri="bar1d.db")
    east_quoter.update()
    print(east_quoter.get_quote(513650))
    print(east_quoter.get_quote(159659))

    xueqiu_quoter = XueQiuQuote(uri="bar1d.db")
    print(xueqiu_quoter.get_quote(513650))
    print(xueqiu_quoter.get_quote(159659))
