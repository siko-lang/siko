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

class Function(SyntaxBase):
    def __init__(self):
        self.name = None
        self.args = []
        self.return_type = None
        self.body = None

class Module(SyntaxBase):
    def __init__(self):
        self.name = None
        self.items = []

class Program(SyntaxBase):
    def __init__(self):
        self.modules = []
    