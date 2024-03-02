import Compiler.Syntax.Base as Base

class Block(Base.SyntaxBase):
    def __init__(self):
        self.statements = []

class Param(Base.SyntaxBase):
    def __init__(self):
        self.name = None
        self.mutable = False
        self.type = None
        self.ownership = None
        self.lifetime = None
        self.dep_lifetimes = []

class Function(Base.SyntaxBase):
    def __init__(self):
        self.module_name = None
        self.name = None
        self.params = []
        self.return_type = None
        self.body = None
        self.ownership_signature = None
        self.return_lifetime = None
        self.return_dep_lifetimes = []
        self.lifetime_dependencies = []
        self.generics = []

    def getAllMembers(self, paths):
        path_members = []
        for p in paths:
            path_members += p.src
            path_members += p.dest
        return self.body.getAllMembers() + self.ownership_signature.members + path_members
