from slaspec.instructions import BitPattern, BitPattern16, Field, TokenFamily, Writer
from slaspec.instructions.vars import Variable


class Pattern:
    def __init__(self, words: int = 1) -> None:
        match words:
            case 1:
                self.bits: BitPattern = ([Field()],)
            case 2:
                self.bits: BitPattern = ([Field()], [Field()])
            case 4:
                self.bits: BitPattern = ([Field()], [Field()], [Field()], [Field()])
            case _:
                raise Exception("Pattern can only be 1, 2, or 4 words long")

    def indexField(self, field: str) -> tuple[int, int]:
        wordIndex: int = -1
        bitIndex: int = -1

        for wi, wv in enumerate(self.bits):
            for bi, bv in enumerate(wv):
                if bv.name == field:
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

    def getField(self, fName: str) -> Field:
        wi, bi = self.indexField(fName)
        return self.bits[wi][bi]

    def setField(self, field: str, val: str) -> None:
        wi, bi = self.indexField(field)
        lenField = len(self.bits[wi][bi].val)
        if lenField == len(val):
            self.bits[wi][bi].val = val

    def setFInt(self, field: str, val: int) -> None:
        wi, bi = self.indexField(field)
        len = self.bits[wi][bi].length()
        self.bits[wi][bi].val = f"{{:0{len}b}}".format(val)[:len]

    def renameField(self, field: str, name: str) -> None:
        wi, bi = self.indexField(field)
        self.bits[wi][bi].name = name

    def signField(self, field: str, signed: bool) -> None:
        wi, bi = self.indexField(field)
        self.bits[wi][bi].sign = signed

    def splitField(self, field: str, fields: BitPattern16) -> None:
        if len(fields) < 2:
            return

        wi, bi = self.indexField(field)

        val = self.bits[wi][bi].val
        sizeFields = sum([len(v.val) for v in fields])

        if len(val) != sizeFields:
            return

        self.bits[wi].pop(bi)

        for v in fields:
            self.bits[wi].insert(bi, v)
            bi += 1


class Instruction:
    def __init__(self, family: "InstructionFamily") -> None:
        self.family: "InstructionFamily" = family
        self.family.addInstr(self)

        self.pattern: Pattern = self.family.pattern
        self.name: str = ""
        self.display: Writer
        self.action: Writer
        self.pcode: Writer
        self.vars: dict[str, Variable] = dict()
        self.multi: bool = False

    def prefix(self) -> str:
        return self.family.prefix


class InstructionFamily:
    def __init__(self) -> None:
        self.name: str = ""
        self.desc: str = ""
        self.prefix: str = ""
        self.pattern: Pattern
        self.instructions: list[Instruction] = []
        self.pcodeops: list[str] = []
        self.tokens: TokenFamily = TokenFamily()
        self.variables: dict[str, set[Variable]] = dict()

    def addInstr(self, instr: Instruction) -> None:
        self.instructions.append(instr)

    def addPcodeOp(self, op: str):
        self.pcodeops.append(op)

    def addVariable(self, var: Variable):
        if var not in self.variables:
            if var.field.name in self.variables:
                self.variables[var.field.name].add(var)
            else:
                self.variables[var.field.name] = set([var])

    def initTokens(self):
        for instr in self.instructions:
            instrToken: TokenFamily = TokenFamily.fromInstr(instr)
            self.tokens = self.tokens.union(instrToken)


class InstructionFamily16(InstructionFamily):
    def __init__(self) -> None:
        super().__init__()
        self.pattern = Pattern(1)


class InstructionFamily32(InstructionFamily):
    def __init__(self) -> None:
        super().__init__()
        self.pattern = Pattern(2)


class InstructionFamily64(InstructionFamily):
    def __init__(self) -> None:
        super().__init__()
        self.pattern = Pattern(4)
