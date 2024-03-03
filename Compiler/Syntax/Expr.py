import Compiler.Syntax.Base as Base

class BoolLiteral(Base.SyntaxBase):
    def __init__(self):
        self.value = None

class FunctionCall(Base.SyntaxBase):
    def __init__(self):
        self.id = None
        self.args = []

class Tuple(Base.SyntaxBase):
    def __init__(self):
        self.args = []

class MemberAccess(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.receiver = None

class MemberCall(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.receiver = None
        self.args = []

class If(Base.SyntaxBase):
    def __init__(self):
        self.cond = None
        self.true_branch = None
        self.false_branch = None

class MatchBranch(Base.SyntaxBase):
    def __init__(self):
        self.pattern = None
        self.body = None

class Match(Base.SyntaxBase):
    def __init__(self):
        self.body = None
        self.branches = []

class Loop(Base.SyntaxBase):
    def __init__(self):
        self.var = None
        self.init = None
        self.body = None

class ForLoop(Base.SyntaxBase):
    def __init__(self):
        self.pattern = None
        self.init = None
        self.body = None

class Break(Base.SyntaxBase):
    def __init__(self):
        self.arg = None

class Continue(Base.SyntaxBase):
    def __init__(self):
        self.arg = None

class Return(Base.SyntaxBase):
    def __init__(self):
        self.arg = None

class VarRef(Base.SyntaxBase):
    def __init__(self):
        self.name = None

class TypeRef(Base.SyntaxBase):
    def __init__(self):
        self.name = None

class StringLiteral(Base.SyntaxBase):
    def __init__(self):
        self.value = None

class BinaryOp(Base.SyntaxBase):
    def __init__(self):
        self.op = None
        self.lhs = None
        self.rhs = None