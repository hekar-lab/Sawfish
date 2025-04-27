from typing import Self
from copy import deepcopy


class BitRange:
    def __init__(self, start: int, end: int):
        self.start = min(start, end)
        self.end = max(start, end)

    def len(self) -> int:
        return self.end - self.start + 1

    def __str__(self) -> str:
        return f"[{self.start, self.end}]"


class Variable:
    import slaspec.registers as regs

    @staticmethod
    def regSuffix(regList: list[str], suffix: str):
        return list(map(lambda reg: reg + "." + suffix, regList))

    REGISTER_SETS = {
        "DREG": regs.DREG,
        "DREGL": regSuffix(regs.DREG, "L"),
        "PREG": regs.PREG,
    }

    def __init__(self, regSet: str) -> None:
        self.regSet: str = regSet

        if regSet not in self.REGISTER_SETS:
            raise IndexError("This set of registers does not exist")

    def registers(self) -> list[str]:
        return self.REGISTER_SETS[self.regSet]


class Mask:
    def __init__(self, val: int):
        self.val = val


class UImm:
    pass


class SImm:
    pass


class Blank:
    pass


type FieldType = Variable | Mask | UImm | SImm | Blank


class Field:
    def __init__(
        self,
        pattern: "Pattern",
        fid: str = "",
        ftype: FieldType = Blank(),
        bitrange: BitRange = BitRange(0, 15),
    ):
        self.pattern = pattern
        self.fid = fid
        self.ftype = ftype
        self.bitrange = bitrange

    def len(self) -> int:
        return self.bitrange.len()

    def tokenName(self) -> str:
        baseName = self.pattern.prefix() + self.fid.capitalize()

        if isinstance(self.ftype, Variable):
            return baseName + self.ftype.regSet
        if isinstance(self.ftype, Mask):
            return baseName
        if isinstance(self.ftype, UImm):
            return baseName + "UImm"
        if isinstance(self.ftype, SImm):
            return baseName + "SImm"
        if isinstance(self.ftype, Blank):
            return "BLANK"

        raise Exception("Unknown FieldType type")

    def _ftypeStr(self):
        if isinstance(self.ftype, Variable):
            return "Var:" + self.ftype.regSet
        if isinstance(self.ftype, Mask):
            return "Mask:" + f"{self.ftype.val:04X}"
        if isinstance(self.ftype, UImm):
            return "UImm"
        if isinstance(self.ftype, SImm):
            return "SImm"
        if isinstance(self.ftype, Blank):
            return "Blank"

    def __str__(self) -> str:
        return f"{self.fid} - [type: {self._ftypeStr()}, range: {self.bitrange}]"


class FieldTemplate:
    def __init__(self, fid: str, ftype: FieldType = Blank(), ln: int = 1):
        self.fid: str = fid
        self.ftype: FieldType = ftype
        self.length: int = ln

    def toField(self, start: int, pattern: "Pattern") -> Field:
        return Field(
            pattern, self.fid, self.ftype, BitRange(start, start + self.length - 1)
        )


type WordPattern = list[Field]
type BytePattern = (
    tuple[WordPattern]
    | tuple[WordPattern, WordPattern]
    | tuple[WordPattern, WordPattern, WordPattern, WordPattern]
)


class Pattern:
    def __init__(self, instrFam: object, words: int = 1) -> None:
        from slaspec.instructions.core import InstructionFamily

        if not isinstance(instrFam, InstructionFamily):
            raise TypeError("Pattern needs an InstructionFamily instance")

        self.instrFam = instrFam
        match words:
            case 1:
                self.bits: BytePattern = ([Field(self)],)
            case 2:
                self.bits: BytePattern = ([Field(self)], [Field(self)])
            case 4:
                self.bits: BytePattern = (
                    [Field(self)],
                    [Field(self)],
                    [Field(self)],
                    [Field(self)],
                )
            case _:
                raise Exception("Pattern can only be 1, 2, or 4 words long")

    def clone(self) -> Self:
        return deepcopy(self)

    def prefix(self) -> str:
        return self.instrFam.prefix

    def indexField(self, fId: str) -> tuple[int, int]:
        wordIndex: int = -1
        bitIndex: int = -1

        for wi, wv in enumerate(self.bits):
            for bi, bv in enumerate(wv):
                if bv.fid == fId:
                    wordIndex = wi
                    bitIndex = bi
                    break
            else:
                continue

            break

        if wordIndex == -1 or bitIndex == -1:
            raise IndexError("Field does not exist in the current pattern")
        else:
            return (wordIndex, bitIndex)

    def getField(self, fId: str) -> Field:
        wi, bi = self.indexField(fId)
        return self.bits[wi][bi]

    def setFieldType(self, fId: str, ftype: FieldType) -> Self:
        out = self.clone()
        wi, bi = out.indexField(fId)

        currentType = out.bits[wi][bi].ftype
        if isinstance(currentType, Blank):
            out.bits[wi][bi].ftype = ftype
        else:
            raise Exception("Cannot change the type of a field that's not Blank typed")

        return out

    def splitField(self, fId: str, fTemplates: list[FieldTemplate]) -> Self:
        out = self.clone()
        if len(fTemplates) <= 0:
            return out

        wi, bi = out.indexField(fId)

        oldField = out.bits[wi][bi]
        newFieldsLen = sum([ft.length for ft in fTemplates])

        if not isinstance(oldField.ftype, Blank):
            raise Exception("Cannot split a field that's not Blank typed")

        if oldField.len() != newFieldsLen:
            return out

        start = out.bits[wi].pop(bi).bitrange.start

        for ft in fTemplates:
            field = ft.toField(start, self)
            start = field.bitrange.end + 1
            out.bits[wi].insert(bi, field)
            bi += 1

        return out

    def __str__(self) -> str:
        out = ""
        for word in self.bits:
            for field in word:
                out += f"{field}, "
            out = out[:-2]
            out += "\n"
        out = out[:-1]
        return out
