def regNameGen(prefix: str, count: int) -> list[str]:
    return [prefix + str(i) for i in range(count)]


DREG = regNameGen("R", 8)
PREG = regNameGen("P", 6) + ["SP", "FP"]
IREG = regNameGen("I", 4)
MREG = regNameGen("M", 4)
LREG = regNameGen("L", 4)
BREG = regNameGen("B", 4)

ACC = regNameGen("A", 2)

LOOPREG = ["LC0", "LB0", "LT0", "LC1", "LB1", "LT1", "CYCLES", "CYCLES2"]
