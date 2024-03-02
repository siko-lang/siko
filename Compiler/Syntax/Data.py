import Compiler.Syntax.Base as Base

class Enum(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.variants = []

class Variant(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.items = []

class Field(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.type = None
        self.lifetime = None
        self.dep_lifetimes = None

class Class(Base.SyntaxBase):
    def __init__(self):
        self.module_name = None
        self.name = None
        self.fields = []
        self.methods = []
        self.derives = []
        self.lifetimes = []
