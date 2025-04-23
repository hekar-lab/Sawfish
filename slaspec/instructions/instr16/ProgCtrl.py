from instructions import Field
from instructions.core import Instruction, InstructionFamily16
from instructions.util import simpleWriter
from instructions.pcode import pCOPY, pMACRO, pOP, pPTR, pRETURN, pVAR
from slaspec.const import IMASK
from slaspec.instructions.vars import Variable


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

        def code(retReg: str) -> str:
            return pOP(pRETURN(retReg))

        self.pcode = code


class RTSInstr(ReturnInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        self.pattern.setFInt("reg", 0x0)
        self.display = simpleWriter("RTS")
        self.pcode = lambda: self.pcode("RETS")


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
        op = "idle"
        self.family.addPcodeOp(op)

        self.pattern.setFInt("reg", 0x0)
        self.display = simpleWriter("IDLE")
        self.pcode = lambda: pOP(pMACRO(op))


class RegInstr(Instruction):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        pat = [Field("regH", 0x0, 1), Field("regL", ln=3)]
        self.pattern.splitField("reg", pat)


class DRegInstr(RegInstr):
    def __init__(self, family: ProgCtrlFamily) -> None:
        super().__init__(family)
        var = Variable(self, self.pattern.getField("regL"), "DREG")
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
        self.display = simpleWriter("CLI")

        def code() -> str:
            ops = pOP(pCOPY(pVAR(self.vars["PReg"]), pPTR(IMASK, 4)))
            ops += pOP(pCOPY(pPTR(IMASK, 4), "0"))
            return ops

        self.pcode = code
