
class GenericVarDeclaration(object):
    def __init__(self, name):
        self.name = name
        self.deps = []

class GenericConstraint(object):
    def __init__(self):
        self.constraint = None

class GenericDeclaration(object):
    def __init__(self):
        self.generics = []
        self.constraints = []

class Named(object):
    def __init__(self, name, args):
        self.name = name
        self.args = args

    def __str__(self) -> str:
        if len(self.args) == 0:
            return "%s" % self.name
        else:
            args = []
            for a in self.args:
                args.append(str(a))
            return "%s[%s]" % (self.name, ", ".join(args))

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        if isinstance(other, Named):
            return self.name == other.name and self.args == other.args
        else:
            return False

    def __hash__(self) -> int:
        return self.name.__hash__()

class Function(object):
    def __init__(self, args, result):
        self.params = args
        self.result = result

    def __str__(self) -> str:
        args = []
        for a in self.params:
            args.append(str(a))
        return "fn(%s) -> %s" % (", ".join(args), self.result)

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        if isinstance(other, Function):
            return self.result == other.result and self.params == other.params
        else:
            return False

    def __hash__(self) -> int:
        return self.result.__hash__()

class Tuple(object):
    def __init__(self, items):
        self.items = items

    def __str__(self):
        items = []
        for i in self.items:
            items.append(str(i))
        return "(%s)" % ", ".join(items)

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        if isinstance(other, Tuple):
            return self.items == other.items
        else:
            return False

    def __hash__(self) -> int:
        return self.items.__hash__()

class Var(object):
    def __init__(self, name):
        self.name = name

    def __str__(self):
        return "%s" % self.name

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        if isinstance(other, Var):
            return self.name == other.name
        else:
            return False

    def __hash__(self) -> int:
        return self.name.__hash__()

class Type(object):
    def __init__(self, kind):
        self.kind = kind

    def __str__(self) -> str:
        return self.kind.__str__()

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        return self.kind == other.kind
    
    def __hash__(self) -> int:
        return self.kind.__hash__()
