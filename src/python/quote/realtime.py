import datetime as dt
from bktrader import datatype
import httpx


class EastBar:
    def __init__(self):
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

    def update(self) -> dict:
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
            afternoon_start = dt.datetime.combine(update_time.date(), dt.time(13, 0, 0))
            time_ratio = (update_time - afternoon_start + dt.timedelta(hours=2)) / dt.timedelta(hours=4)
            predicted_today_volume = record["f5"] / time_ratio
            predicted_today_amount = record["f6"] / time_ratio

            code = int(record["f12"])
            bar_dict[code] = datatype.Bar(
                code=code,
                dt=self.days_since_epoch(record["f297"]),
                preclose=record["f18"],
                open=record["f17"],
                high=record["f15"],
                low=record["f16"],
                close=record["f2"],
                volume=predicted_today_volume,
                amount=predicted_today_amount,
            )

        return bar_dict


if __name__ == "__main__":
    east_bar = EastBar()
    bar_dict = east_bar.update()
    print(bar_dict)
