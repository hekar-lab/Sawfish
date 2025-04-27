from slaspec.instructions.pattern import (
    Pattern,
    Variable,
    Field,
    FieldTemplate,
    FieldType,
    SImm,
)


class Token:
    def __init__(self, name: str, start: int, end: int, signed: bool) -> None:
        self.name: str = name
        self.start: int = start
        self.end: int = end
        self.signed: bool = signed

    def __eq__(self, value: object, /) -> bool:
        return (
            isinstance(value, Token)
            and value.name == self.name
            and value.start == self.start
            and value.end == self.end
            and value.signed == self.signed
        )

    def _len(self) -> int:
        return abs(self.start - self.end)

    def __lt__(self, other: object, /) -> bool:
        if not isinstance(other, Token):
            raise TypeError(
                f"'<' not supported between instances of 'Token' and '{type(other)}'"
            )

        return (
            self.start < other.start
            or (self.start == other.start and self._len() < other._len())
            or (
                self.start == other.start
                and self._len() == other._len()
                and self.name < other.name
            )
        )

        return False

    def __hash__(self) -> int:
        return hash((self.name, self.start, self.end, self.signed))

    def __str__(self) -> str:
        return (
            f"({self.name} | [{self.start}, {self.end}] {'-' if self.signed else '+'})"
        )

    @staticmethod
    def fromField(field: Field) -> "Token":
        return Token(
            field.tokenName(),
            field.bitrange.start,
            field.bitrange.end,
            isinstance(field.ftype, SImm),
        )


class TokenVar:
    def __init__(self, name: str, regs: list[str]):
        self.name: str = name
        self.regs: list[str] = regs.copy()

    def __eq__(self, value: object, /) -> bool:
        return isinstance(value, TokenVar) and value.name == self.name

    def __hash__(self) -> int:
        return hash(self.name)

    def regvar(self) -> str:
        return " ".join(self.regs)

    @staticmethod
    def fromField(field: Field) -> "TokenVar":
        if not isinstance(field.ftype, Variable):
            raise Exception(
                "Cannot create a TokenVar from a Field that's not Variable typed"
            )

        return TokenVar(field.tokenName(), field.ftype.registers())


type TokenSet = set[Token]


class TokenFamily:
    def __init__(self) -> None:
        self.tsets: tuple[TokenSet, TokenSet, TokenSet, TokenSet] = (
            set(),
            set(),
            set(),
            set(),
        )

    def union(self, fam: "TokenFamily") -> "TokenFamily":
        retFam = TokenFamily()
        retFam.tsets = (
            self.tsets[0].union(fam.tsets[0]),
            self.tsets[1].union(fam.tsets[1]),
            self.tsets[2].union(fam.tsets[2]),
            self.tsets[3].union(fam.tsets[3]),
        )
        return retFam


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

    def getField(self, fId: str):
        return self.pattern.getField(fId)

    def splitField(self, fId: str, fTemplates: list[FieldTemplate]):
        self.pattern = self.pattern.splitField(fId, fTemplates)

    def setFieldType(self, fId: str, ftype: FieldType):
        self.pattern = self.pattern.setFieldType(fId, ftype)


class InstructionFamily:
    def __init__(self) -> None:
        self.name: str = ""
        self.desc: str = ""
        self.prefix: str = ""
        self.pattern: Pattern
        self.instructions: list[Instruction] = []
        self.pcodeops: list[str] = []
        self.tfam: TokenFamily = TokenFamily()
        self.varAttach: set[TokenVar] = set()

    def addInstr(self, instr: Instruction) -> None:
        self.instructions.append(instr)

    def addPcodeOp(self, op: str):
        self.pcodeops.append(op)

    def initTokens(self):
        for instr in self.instructions:
            wi = 0
            for word in instr.pattern.bits:
                for field in word:
                    self.tfam.tsets[wi].add(Token.fromField(field))
                    if isinstance(field.ftype, Variable):
                        self.varAttach.add(TokenVar.fromField(field))

                wi += 1

    def splitField(self, fId: str, fTemplates: list[FieldTemplate]):
        self.pattern = self.pattern.splitField(fId, fTemplates)

    def setFieldType(self, fId: str, ftype: FieldType):
        self.pattern = self.pattern.setFieldType(fId, ftype)


class InstructionFamily16(InstructionFamily):
    def __init__(self) -> None:
        super().__init__()
        self.pattern = Pattern(self, 1)


class InstructionFamily32(InstructionFamily):
    def __init__(self) -> None:
        super().__init__()
        self.pattern = Pattern(self, 2)


class InstructionFamily64(InstructionFamily):
    def __init__(self) -> None:
        super().__init__()
        self.pattern = Pattern(self, 4)
