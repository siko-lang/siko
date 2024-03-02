import Compiler.Syntax.Base as Base

class Trait(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.generic_parameters = []
        self.dependent_parameters = []
        self.declarations = []
        self.methods = []
