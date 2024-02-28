class SerializableMixin:
    def to_dict(self):
        return {key: value for key, value in self.__dict__.items()}

def default_serializer(obj):
    if isinstance(obj, SerializableMixin):
        return obj.to_dict()
    raise TypeError(f"Object of type {obj.__class__.__name__} is not JSON serializable")

class SyntaxBase(SerializableMixin):
    def __init__(self):
        pass

class BoolLiteral(SyntaxBase):
    def __init__(self):
        self.value = None

class Import(SyntaxBase):
    def __init__(self):
        self.module = None
        self.alias = None

class FunctionCall(SyntaxBase):
    def __init__(self):
        self.id = None
        self.args = []

class MemberAccess(SyntaxBase):
    def __init__(self):
        self.name = None
        self.receiver = None

class MemberCall(SyntaxBase):
    def __init__(self):
        self.name = None
        self.receiver = None
        self.args = []

class If(SyntaxBase):
    def __init__(self):
        self.cond = None
        self.true_branch = None
        self.false_branch = None

class Loop(SyntaxBase):
    def __init__(self):
        self.var = None
        self.init = None
        self.body = None

class Break(SyntaxBase):
    def __init__(self):
        self.arg = None

class Continue(SyntaxBase):
    def __init__(self):
        self.arg = None

class Return(SyntaxBase):
    def __init__(self):
        self.arg = None

class VarRef(SyntaxBase):
    def __init__(self):
        self.name = None

class TypeRef(SyntaxBase):
    def __init__(self):
        self.name = None

class LetStatement(SyntaxBase):
    def __init__(self):
        self.var_name = None
        self.rhs = None

class ExprStatement(SyntaxBase):
    def __init__(self):
        self.expr = None
        self.requires_semicolon = False
        self.has_semicolon = False

class Block(SyntaxBase):
    def __init__(self):
        self.statements = []

class Type(SyntaxBase):
    def __init__(self):
        self.name = None

class Arg(SyntaxBase):
    def __init__(self):
        self.name = None
        self.type = None
        self.ownership = None
        self.lifetime = None
        self.dep_lifetimes = []

class Function(SyntaxBase):
    def __init__(self):
        self.module_name = None
        self.name = None
        self.args = []
        self.return_type = None
        self.body = None
        self.ownership_signature = None
        self.return_lifetime = None
        self.return_dep_lifetimes = []
        self.lifetime_dependencies = []

    def getAllMembers(self, paths):
        path_members = []
        for p in paths:
            path_members += p.src
            path_members += p.dest
        return self.body.getAllMembers() + self.ownership_signature.members + path_members

class Enum(object):
    def __init__(self):
        self.name = None
        self.variants = []

class Variant(object):
    def __init__(self):
        self.name = None
        self.items = []

class Field(object):
    def __init__(self):
        self.name = None
        self.type = None
        self.lifetime = None
        self.dep_lifetimes = None

class Class(object):
    def __init__(self):
        self.module_name = None
        self.name = None
        self.fields = []
        self.methods = []
        self.derives = []
        self.lifetimes = []

class Module(SyntaxBase):
    def __init__(self):
        self.name = None
        self.items = []

class Program(SyntaxBase):
    def __init__(self):
        self.modules = []
        self.functions = {}
        self.classes = {}
    