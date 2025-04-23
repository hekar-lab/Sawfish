from pathlib import Path

from slaspec.instructions import InstructionFamily16
from slaspec.instructions.instr16.NOP16 import NOP16Family
from slaspec.instructions.instr16.ProgCtrl import ProgCtrlFamily


class SLASpecBuilder:
    def __init__(self) -> None:
        self.path = "blackfinplus.slaspec"

        self.instrFam16: list[InstructionFamily16] = [NOP16Family(), ProgCtrlFamily()]

    def build(self, path: Path) -> None:
        sincPath: Path = path.joinpath("includes")
        for instrFam in self.instrFam16:
            out = ""
            path16: Path = sincPath.joinpath(f"instr16/{instrFam.name}.sinc")

            # Tokens
            tokWord = 16
            for tokens in instrFam.tokens:
                out += f"\ndefine token {instrFam.prefix}Inst{tokWord} (16)"
                for tok in tokens:
                    out += f"\n    {tok.name} = ({tok.start}, {tok.end})"
                    if tok.signed:
                        out += " signed"
                out += "\n;"
                tokWord += 16

            # Variables
            out += "\n"
            for varList in instrFam.variables.values():
                for var in varList:
                    out += f"\nattach variables [{var.tokenName()}] [{var.regvar()}];"

            # instructions
            out += "\n"
            for instr in instrFam.instructions:
                out += f'\n{instrFam.name}:^"{instr.name}"'

                display = instr.display()
                if display:
                    out += f" {display}"

                action = instr.action()
                if action:
                    out += " ["
                    out += action
                    out += "\n]"

                pcode = instr.pcode()
                if pcode:
                    out += " {"
                    out += pcode
                    out += "\n}"

            path16.parent.mkdir(exist_ok=True, parents=True)
            with path16.open("w") as outfile:
                outfile.write(out)
