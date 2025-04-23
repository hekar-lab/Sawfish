from collections.abc import Callable

from slaspec.instructions.core import Instruction


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
        if self.iterIndex < 4:
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
        instr: Instruction,
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
type Writer = Callable[..., str]
