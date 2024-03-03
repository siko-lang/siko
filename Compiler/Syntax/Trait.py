import Compiler.Syntax.Base as Base

class Trait(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.generics = []
        self.generic_parameters = []
        self.dependent_parameters = []
        self.method_declarations = []
        self.methods = []

class Instance(Base.SyntaxBase):
    def __init__(self):
        self.type = []
        self.generics = []
        self.methods = []
