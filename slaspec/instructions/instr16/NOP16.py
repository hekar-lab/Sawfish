from slaspec.instructions.core import Instruction, InstructionFamily16
from slaspec.instructions.pattern import FieldTemplate as FT, Mask


class NOP16Family(InstructionFamily16):
    def __init__(self) -> None:
        super().__init__()
        self.name = "NOP16"
        self.desc = "16-bit Slot Nop"
        self.prefix = "nop"

        self.splitField("", [FT("sig", Mask(0x0000), 16)])

        # Instruction
        NOPInstrNOP16(self)


class NOPInstrNOP16(Instruction):
    def __init__(self, family: NOP16Family) -> None:
        super().__init__(family)
        self.name = "NOP"
