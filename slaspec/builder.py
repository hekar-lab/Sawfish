from math import ceil
from pathlib import Path

from slaspec.instructions.core import InstructionFamily16
from slaspec.instructions.instr16.NOP16 import NOP16Family
from slaspec.instructions.instr16.ProgCtrl import ProgCtrlFamily
from slaspec.instructions.pattern import Mask


class SLASpecBuilder:
    def __init__(self) -> None:
        self.path = "blackfinplus.slaspec"

        self.instrFam16: list[InstructionFamily16] = [NOP16Family(), ProgCtrlFamily()]

        for fam in self.instrFam16:
            fam.initTokens()

    def build(self, path: Path) -> None:
        sincPath: Path = path.joinpath("includes")
        for instrFam in self.instrFam16:
            out = ""
            path16: Path = sincPath.joinpath(f"instr16/{instrFam.name}.sinc")

            # Tokens
            tokWord = 16
            for tokens in sorted(list(instrFam.tfam.tsets)):
                if not tokens:
                    continue
                out += f"\ndefine token {instrFam.prefix}Inst{tokWord} (16)"

                for tok in sorted(list(tokens)):
                    out += f"\n    {tok.name} = ({tok.start}, {tok.end})"
                    if tok.signed:
                        out += " signed"
                out += "\n;"
                tokWord += 16

            # Variables
            out += "\n"
            for var in instrFam.varAttach:
                out += f"\nattach variables [{var.name}] [{var.regvar()}];"

            # instructions
            out += "\n"
            for instr in instrFam.instructions:
                out += f'\n{instrFam.name}:^"{instr.name}"'

                display = instr.display()
                if display:
                    out += f" {display}"

                out += "\n\tis "

                for word in instr.pattern.bits:
                    for field in word:
                        out += f"{field.tokenName()}"
                        if isinstance(field.ftype, Mask):
                            hexDigits: int = ceil(field.bitrange.len() / 4)
                            out += f"=0x{{:0{hexDigits}x}}".format(field.ftype.val)
                        out += " & "
                    out = out[:-2] + "; "
                out = out[:-2]

                out += "\n"
                action = instr.action()
                if action:
                    out += "["
                    out += action
                    out += "\n] "

                pcode = instr.pcode()
                if pcode:
                    out += "{"
                    out += pcode
                    out += "\n}"

            path16.parent.mkdir(exist_ok=True, parents=True)
            with path16.open("w") as outfile:
                outfile.write(out)
