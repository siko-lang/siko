class InstructionId(object):
    def __init__(self):
        self.block = 0
        self.value = 0

    def prev(self):
        p = InstructionId()
        p.block = self.block
        p.value = self.value - 1
        return p
    
    def __eq__(self, other):
        if isinstance(other, InstructionId):
            return self.block == other.block and self.value == other.value
        else:
            return False

    def __hash__(self):
        return self.block.__hash__()

    def __str__(self):
        return "$%s.%s" % (self.block, self.value)

    def __repr__(self) -> str:
        return self.__str__()

class BaseInstruction(object):
    def __init__(self):
        self.id = 0
        self.tv_info = None
        self.members = []
        self.moves = []
        self.type = None
        self.type_signature = None
        self.ownership = None

class BlockRef(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.value = 0

    def __str__(self):
        return "block ref: #%s" % self.value

class NamedFunctionCall(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.ctor = False
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        if self.ctor:
            return "%s(%s)/ctor" % (self.name, ", ".join(args))
        else:
            return "%s(%s)" % (self.name, ", ".join(args))

class Tuple(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        return "(%s)" % (", ".join(args))

class DynamicFunctionCall(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.callable = None
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        return "%s(%s)" % (self.callable, ", ".join(args))

class MethodCall(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.receiver = None
        self.name = None
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        return "%s.%s(%s)" % (self.receiver, self.name, ", ".join(args))

class Bind(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.rhs = None

    def __str__(self):
        return "%s = %s" % (self.name, self.rhs)

class MemberAccess(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.receiver = None
        self.name = None
        self.index = 0
        
    def __str__(self):
        return "%s.%s" % (self.receiver, self.name)

class ValueRef(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.bind_id = None
        self.fields = []
        self.indices = []
        self.borrow = False
        self.move = False
        self.clone = False

    def __str__(self):
        if len(self.fields) > 0:
            fields = ".".join(self.fields)
            return "%s.%s/%s" % (self.name, fields, self.bind_id)
        else:
            return "%s/%s" % (self.name, self.bind_id)

class DropVar(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.cancelled = False
        
    def __str__(self):
        if self.cancelled:
            return "drop(%s)/Inactive" % (self.name)
        else:
            return "drop(%s)/Active" % (self.name)

class Nop(BaseInstruction):
    def __init__(self):
        super().__init__()

    def __str__(self):
        return "nop"

class If(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.cond = None
        self.true_branch = None
        self.false_branch = None

    def __str__(self):
        if self.false_branch:
            return "if %s then %s else %s" % (self.cond, self.true_branch, self.false_branch)
        else:
            return "if %s then %s" % (self.cond, self.true_branch)

class Loop(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.var = None
        self.init = None
        self.body = None

    def __str__(self):
        return "loop %s = %s %s" % (self.var, self.init, self.body)

class Break(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None

    def __str__(self):
        return "break %s" % self.arg

class Continue(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None
    
    def __str__(self):
        return "continue %s" % self.arg

class Return(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None
    
    def __str__(self):
        return "return %s" % self.arg

class BoolLiteral(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.value = None
    
    def __str__(self):
        return "bool %s" % self.value
