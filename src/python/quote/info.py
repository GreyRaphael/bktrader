import duckdb


def query_info_all(uri: str) -> dict:
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute("SELECT code,name,mer,cer FROM info").fetchall()
    return {code: (name, mer, cer) for code, name, mer, cer in records}


def query_sector_codes(uri: str, sectors: list[int]) -> list[int]:
    placeholders = ",".join([str(s) for s in sectors])
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute(f"SELECT info.code FROM info JOIN bar1d ON info.code=bar1d.code WHERE info.sector IN ({placeholders}) AND bar1d.dt = (SELECT MAX(bar1d.dt) FROM bar1d)").fetchall()
    return [record[0] for record in records]
