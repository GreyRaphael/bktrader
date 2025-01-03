import duckdb


def query_info_all(uri: str) -> dict:
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute("SELECT code,name,mer,cer FROM info").fetchall()
    return {code: (name, mer, cer) for code, name, mer, cer in records}


def query_sector_codes(uri: str, sectors: list[int]) -> list[int]:
    placeholders = ",".join([str(s) for s in sectors])
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute(f"SELECT code FROM info WHERE sector IN ({placeholders})").fetchall()
    return [record[0] for record in records]
