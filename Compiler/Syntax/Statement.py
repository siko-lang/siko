import Compiler.Syntax.Base as Base

class LetStatement(Base.SyntaxBase):
    def __init__(self):
        self.pattern = None
        self.rhs = None

class ExprStatement(Base.SyntaxBase):
    def __init__(self):
        self.expr = None
        self.requires_semicolon = False
        self.has_semicolon = False

class AssignStatement(Base.SyntaxBase):
    def __init__(self):
        self.lhs = None
        self.rhs = None
        
