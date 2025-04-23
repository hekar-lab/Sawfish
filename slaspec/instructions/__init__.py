class Field:
    def __init__(
        self, name: str = "x", v: int = -1, ln: int = 16, s: bool = False
    ) -> None:
        strVal = "x" * ln if v < 0 else f"{{:0{ln}b}}".format(v)
        self.name = name
        self.val = strVal
        self.sign = s

    def length(self) -> int:
        return len(self.val)

    def tokenName(self, instr: "Instruction") -> str:
        return instr.prefix() + self.name.title() + "S" if self.sign else "U"


class Token:
    def __init__(self, name: str, start: int, end: int, signed: bool) -> None:
        self.name = name
        self.start = start
        self.end = end
        self.signed = signed

    def __eq__(self, value: object, /) -> bool:
        return (
            isinstance(value, Token)
            and value.name == self.name
            and value.start == self.start
            and value.end == self.end
            and value.signed == self.signed
        )

    def __hash__(self) -> int:
        return hash((self.name, self.start, self.end, self.signed))


type TokenSet = set[Token]


class TokenFamily:
    def __init__(self) -> None:
        self.iterIndex = -1
        self.tokens: tuple[TokenSet, TokenSet, TokenSet, TokenSet] = (
            set(),
            set(),
            set(),
            set(),
        )

    def __iter__(self) -> "TokenFamily":
        return self

    def __next__(self) -> TokenSet:
        if self.iterIndex < 3:
            self.iterIndex += 1
            return self.tokens[self.iterIndex]
        else:
            self.iterIndex = -1
            raise StopIteration

    def union(self, fam: "TokenFamily") -> "TokenFamily":
        retFam = TokenFamily()
        retFam.tokens = (
            self.tokens[0].union(fam.tokens[0]),
            self.tokens[1].union(fam.tokens[1]),
            self.tokens[2].union(fam.tokens[2]),
            self.tokens[3].union(fam.tokens[3]),
        )
        return retFam

    @staticmethod
    def fromInstr(
        instr: "Instruction",
    ) -> "TokenFamily":
        fam: TokenFamily = TokenFamily()

        for wi, word in enumerate(instr.pattern.bits):
            start = 0
            for field in word:
                end = start + field.length() - 1
                fam.tokens[wi].add(
                    Token(field.tokenName(instr), start, end, field.sign)
                )

                for var in instr.family.variables[field.name]:
                    fam.tokens[wi].add(Token(var.tokenName(), start, end, False))
                start = end + 1

        return fam


type BitPattern16 = list[Field]
type BitPattern = (
    tuple[BitPattern16]
    | tuple[BitPattern16, BitPattern16]
    | tuple[BitPattern16, BitPattern16, BitPattern16, BitPattern16]
)


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


def regSuffix(regList: list[str], suffix: str):
    return list(map(lambda reg: reg + "." + suffix, regList))


class Variable:
    import slaspec.registers as regs

    REGISTER_SETS = {
        "DREG": regs.DREG,
        "DREGL": regSuffix(regs.DREG, "L"),
        "PREG": regs.PREG,
    }

    def __init__(self, instr: "Instruction", field: Field, regSet: str) -> None:
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


class Instruction:
    def __init__(self, family: "InstructionFamily") -> None:
        self.family: "InstructionFamily" = family
        self.family.addInstr(self)

        self.pattern: Pattern = self.family.pattern
        self.name: str = ""
        self.vars: dict[str, Variable] = dict()
        self.multi: bool = False

    def prefix(self) -> str:
        return self.family.prefix

    def display(self) -> str:
        return ""

    def action(self) -> str:
        return ""

    def pcode(self) -> str:
        return ""


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
