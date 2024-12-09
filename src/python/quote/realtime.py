import math
import datetime as dt
import httpx
import duckdb
from bktrader import datatype


class EastQuote:
    def __init__(self, uri: str):
        self.uri = uri
        self.client = httpx.Client(
            headers={
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/115.0",
            }
        )
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

    def days_since_epoch(self, value: int) -> int:
        year = value // 10000  # 2024
        month = (value % 10000) // 100  # 12
        day = value % 100  # 06
        delta = dt.date(year, month, day) - dt.date(1970, 1, 1)
        return delta.days

    def update(self):
        fields = ",".join(self.mapping.keys())
        timestamp = int(dt.datetime.now().timestamp() * 1000)
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
            "fields": fields,
            "_": timestamp,
        }
        rsp = self.client.get("http://push2.eastmoney.com/api/qt/clist/get", params=url_params, timeout=10).json()

        bar_dict = {}
        for record in rsp["data"]["diff"]:
            update_time = dt.datetime.fromtimestamp(record["f124"])  # run after 14:00

            morning_start = dt.datetime.combine(update_time.date(), dt.time(9, 30, 0))
            morning_end = dt.datetime.combine(update_time.date(), dt.time(11, 30, 0))
            afternoon_start = dt.datetime.combine(update_time.date(), dt.time(13, 0, 0))
            afternoon_end = dt.datetime.combine(update_time.date(), dt.time(15, 0, 0))
            if update_time < morning_start:
                return {}
            elif update_time <= morning_end:
                time_ratio = (update_time - morning_start) / dt.timedelta(hours=4)
            elif update_time < afternoon_start:
                time_ratio = 0.5
            elif update_time <= afternoon_end:
                time_ratio = (update_time - afternoon_start + dt.timedelta(hours=2)) / dt.timedelta(hours=4)
            else:
                time_ratio = 1.0

            predicted_today_volume = record["f5"] * 1e2 / time_ratio  # 1e2单位手换算成股
            predicted_today_amount = record["f6"] / time_ratio

            code = int(record["f12"])
            bar_dict[code] = (
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

        self.realtime_bars = bar_dict

    def get_quote(self, code: int) -> datatype.Bar:
        with duckdb.connect(self.uri, read_only=True) as conn:
            query = "SELECT ROUND(close/1e4, 3), adjfactor FROM etf WHERE code=? ORDER BY dt DESC LIMIT 1"
            last_close, factor = conn.execute(query, [code]).fetchone()
        code, dt, preclose, open, high, low, last, predicted_vol, predicted_amt = self.realtime_bars[code]
        adjfactor = factor if math.isclose(last_close, preclose) else last_close / preclose * factor
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


if __name__ == "__main__":
    east_bar = EastQuote(uri="bar1d.db")
    east_bar.update()
    print(east_bar.get_quote(513650))
    print(east_bar.get_quote(159659))
