from slaspec.instructions.pattern import FieldTemplate as FT, Mask, Variable
from slaspec.instructions.core import Instruction, InstructionFamily16
from slaspec.instructions.pcode import pCOPY, pMACRO, pOP, pPTR, pRETURN, pVAR
from slaspec.const import IMASK


class ProgCtrlFamily(InstructionFamily16):
    def __init__(self) -> None:
        super().__init__()
        self.name = "ProgCtrl"
        self.desc = "Basic Program Sequencer Control Functions"
        self.prefix = "pgc"

        ftemp = [FT("sig", Mask(0x00), 8), FT("opc", ln=4), FT("reg", ln=4)]
        self.splitField("", ftemp)
        # self.variables = []
        # self.pcodeops = ["idle", "csync", "ssync", "emuexcpt", "raise", "excpt"]

        # Instructions
        RTSInstr(self)
        IdleInstr(self)
        CliInstr(self)


class ReturnInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.name = "Return"
        self.setFieldType("opc", Mask(0x01))

    def pcode(self, retReg: str = "") -> str:
        return pOP(pRETURN(retReg))


class RTSInstr(ReturnInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.setFieldType("reg", Mask(0x00))

    def display(self) -> str:
        return '"RTS"'

    def pcode(self, retReg: str = "RETS") -> str:
        return super().pcode(retReg)


class SyncModeInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.setFieldType("opc", Mask(0x02))


class SyncInstr(SyncModeInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.name = "Sync"


class IdleInstr(SyncInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.op = "idle"
        self.family.addPcodeOp(self.op)

        self.setFieldType("reg", Mask(0x00))

    def display(self) -> str:
        return '"IDLE"'

    def pcode(self) -> str:
        return pOP(pMACRO(self.op))


class RegInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.setFieldType("opc", Mask(0x03))
        fTemp = [FT("regH", Mask(0x0)), FT("regL", ln=3)]
        self.splitField("reg", fTemp)


class DRegInstr(RegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        var = Variable("DREG")
        self.setFieldType("regL", var)


class PRegInstr(RegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        var = Variable("PREG")
        self.setFieldType("regL", var)


class IMaskInstr(DRegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.name = "IMaskMv"


class CliInstr(IMaskInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)

    def display(self) -> str:
        return '"CLI"'

    def pcode(self) -> str:
        ops = pOP(
            pCOPY(
                pVAR(self.getField("regL").tokenName()),
                pPTR(IMASK, 4),
            )
        )
        ops += pOP(pCOPY(pPTR(IMASK, 4), "0"))

        return ops
