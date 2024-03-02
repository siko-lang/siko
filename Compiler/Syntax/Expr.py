import Compiler.Syntax.Base as Base

class BoolLiteral(Base.SyntaxBase):
    def __init__(self):
        self.value = None

class FunctionCall(Base.SyntaxBase):
    def __init__(self):
        self.id = None
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

class Loop(Base.SyntaxBase):
    def __init__(self):
        self.var = None
        self.init = None
        self.body = None

class ForLoop(Base.SyntaxBase):
    def __init__(self):
        self.var = None
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
