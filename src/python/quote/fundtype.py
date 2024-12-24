import enum


class ETFType(str, enum.Enum):
    qdii = "qdii"
    commodity = "commodity"
    bond = "bond"
    all = "all"


class LOFType(str, enum.Enum):
    qdii = "qdii"
    commodity = "commodity"  # alternative investment
    bond = "bond"
    all = "all"
