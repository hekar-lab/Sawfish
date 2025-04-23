from instructions import Writer


def simpleWriter(txt: str) -> Writer:
    return lambda: txt
