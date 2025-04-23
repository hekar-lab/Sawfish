from slaspec.instructions import Field
from slaspec.instructions.core import Instruction


def regSuffix(regList: list[str], suffix: str):
    return list(map(lambda reg: reg + "." + suffix, regList))


class Variable:
    import slaspec.registers as regs

    REGISTER_SETS = {
        "DREG": regs.DREG,
        "DREGL": regSuffix(regs.DREG, "L"),
        "PREG": regs.PREG,
    }

    def __init__(self, instr: Instruction, field: Field, regSet: str) -> None:
        self.instr: Instruction = instr
        self.field: Field = field
        self.regSet: str = regSet

        if regSet not in self.REGISTER_SETS:
            raise IndexError("This set of registers does not exist")

    def tokenName(self) -> str:
        return self.instr.prefix() + self.field.name.capitalize() + self.regSet

    def registers(self) -> list[str]:
        return self.REGISTER_SETS[self.regSet]

    def regvar(self) -> str:
        return " ".join(self.registers())

    def __eq__(self, value: object, /) -> bool:
        return isinstance(value, Variable) and value.tokenName() == self.tokenName()

    def __hash__(self) -> int:
        return hash(self.tokenName())
