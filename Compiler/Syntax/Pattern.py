import Compiler.Syntax.Base as Base

class Wildcard(Base.SyntaxBase):
    def __init__(self):
        pass

class Named(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.args = []

class Bind(Base.SyntaxBase):
    def __init__(self):
        self.name = None

class Tuple(Base.SyntaxBase):
    def __init__(self):
        self.args = []
