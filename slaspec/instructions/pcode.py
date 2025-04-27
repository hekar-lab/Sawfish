from slaspec.const import DEFAULT_MEM


def pOP(code: str) -> str:
    return f"\n\t{code};"


def pMACRO(macro: str) -> str:
    return f"{macro}()"


def pLOCAL(var: str, size: int = 1) -> str:
    return f"local {var}:{size}"


def pVAR(var: str) -> str:
    return var


def pCOPY(var: str, val: str) -> str:
    return f"{var} = {val}"


def pPTR(addr: str, size: int = 1, mem: str = DEFAULT_MEM) -> str:
    return f"*[{mem}]:{size} {addr}"


def pRETURN(addr: str) -> str:
    return f"return [{addr}]"
