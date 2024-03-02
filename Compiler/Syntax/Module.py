import Compiler.Syntax.Base as Base

class Import(Base.SyntaxBase):
    def __init__(self):
        self.module = None
        self.alias = None

class Module(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.items = []
