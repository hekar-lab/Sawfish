from instructions.core import Instruction, InstructionFamily16


class NOP16Family(InstructionFamily16):
    def __init__(self) -> None:
        super().__init__()
        self.name = "NOP16"
        self.desc = "16-bit Slot Nop"
        self.prefix = "nop"

        self.pattern.renameField("x", "sig")
        self.pattern.setFInt("sig", 0x0000)

        # Instruction
        NOPInstrNOP16(self)


class NOPInstrNOP16(Instruction):
    def __init__(self, family: NOP16Family) -> None:
        super().__init__(family)
        self.name = "NOP"
