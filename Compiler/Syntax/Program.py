import Compiler.Syntax.Base as Base

class Program(Base.SyntaxBase):
    def __init__(self):
        self.modules = []
        self.functions = {}
        self.classes = {}
    