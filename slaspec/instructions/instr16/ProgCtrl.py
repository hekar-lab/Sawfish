from slaspec.instructions import Field
from slaspec.instructions import Instruction, InstructionFamily16
from slaspec.instructions.pcode import pCOPY, pMACRO, pOP, pPTR, pRETURN, pVAR
from slaspec.const import IMASK
from slaspec.instructions import Variable
import inspect


class ProgCtrlFamily(InstructionFamily16):
    def __init__(self) -> None:
        super().__init__()
        self.name = "ProgCtrl"
        self.desc = "Basic Program Sequencer Control Functions"
        self.prefix = "pgc"

        pat = [Field("sig", 0x00, 8), Field("opc", ln=4), Field("reg", ln=4)]
        self.pattern.splitField("x", pat)
        # self.variables = []
        # self.pcodeops = ["idle", "csync", "ssync", "emuexcpt", "raise", "excpt"

        # Instructions
        RTSInstr(self)
        IdleInstr(self)
        CliInstr(self)


class ReturnInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.name = "Return"
        self.pattern.setFInt("opc", 0x1)

    def pcode(self, retReg: str = "") -> str:
        return pOP(pRETURN(retReg))


class RTSInstr(ReturnInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.pattern.setFInt("reg", 0x0)
        print(inspect.getsource(self.pcode))

    def display(self) -> str:
        return "RTS"

    def pcode(self, retReg: str = "RETS") -> str:
        return super().pcode(retReg)


class SyncModeInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.pattern.setFInt("opc", 0x2)


class SyncInstr(SyncModeInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.name = "Sync"


class IdleInstr(SyncInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.op = "idle"
        self.family.addPcodeOp(self.op)

        self.pattern.setFInt("reg", 0x0)

    def display(self) -> str:
        return "IDLE"

    def pcode(self) -> str:
        return pOP(pMACRO(self.op))


class RegInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        pat = [Field("regH", 0x0, 1), Field("regL", ln=3)]
        self.pattern.splitField("reg", pat)


class DRegInstr(RegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        var = Variable(self, self.pattern.getField("regL"), "DREG")
        self.vars["DReg"] = var
        self.family.addVariable(var)


class PRegInstr(RegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        var = Variable(self, self.pattern.getField("regL"), "PREG")
        self.vars["PReg"] = var
        self.family.addVariable(var)


class IMaskInstr(DRegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.name = "IMaskMv"


class CliInstr(IMaskInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)

    def display(self) -> str:
        return "CLI"

    def pcode(self) -> str:
        ops = pOP(pCOPY(pVAR(self.vars["DReg"]), pPTR(IMASK, 4)))
        ops += pOP(pCOPY(pPTR(IMASK, 4), "0"))

        return ops
