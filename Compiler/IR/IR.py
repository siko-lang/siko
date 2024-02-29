import Compiler.IR.Instruction as Instruction

class TempVar(object):
    def __init__(self):
        self.value = 0
        self.arg = False

    def __str__(self):
        if self.arg:
            return "$arg_%s" % self.value
        else:
            return "$tmp_%s" % self.value

    def __repr__(self):
        return self.__str__()

    def __eq__(self, other):
        if isinstance(other, TempVar):
            return self.value == other.value and self.arg == other.arg
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

class Body(object):
    def __init__(self):
        self.blocks = []

    def dump(self):
        for b in self.blocks:
           print("#%d. block:" % b.id)
           b.dump()

    def getFirst(self):
        for b in self.blocks:
            if b.id == 0:
                return b
        return None

    def getInstruction(self, id):
        return self.blocks[id.block].instructions[id.value]

    def getAllMembers(self):
        members = []
        for b in self.blocks:
            for i in b.instructions:
                for m in i.members:
                    members.append(m)
        return members

    def getBlock(self, blockref):
        value = blockref
        if isinstance(blockref, Instruction.BlockRef):
            value = blockref.value
        for b in self.blocks:
            if b.id == value:
                return b
        return None

class Block(object):
    def __init__(self):
        self.id = None
        self.instructions = []

    def addInstruction(self, instruction):
        index = len(self.instructions)
        id = Instruction.InstructionId()
        id.value = index
        id.block = self.id
        instruction.id = id
        self.instructions.append(instruction)
        return id

    def getLast(self):
        return self.instructions[-1]

    def getLastReal(self):
        last = None
        for i in self.instructions:
            if isinstance(i, Instruction.DropVar):
                continue
            if isinstance(i, Instruction.Nop):
                continue
            last = i
        return last

    def dump(self):
        for (index, i) in enumerate(self.instructions):
           print("$%d. %s" % (index, i))

class Program(object):
    def __init__(self):
        self.modules = []
        self.functions = {}
        self.classes = {}
