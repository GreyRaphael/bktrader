import duckdb


def query_info_all(uri: str) -> dict:
    with duckdb.connect(uri, read_only=True) as conn:
        records = conn.execute("SELECT code,name,mer,cer FROM info").fetchall()
    return {code: (name, mer, cer) for code, name, mer, cer in records}
