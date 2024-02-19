class FunctionInstantiationSignature(object):
    def __init__(self):
        self.name = None
        self.args = []
        self.result = []
        self.members = []

    def __str__(self):
        return "(%s/%s/%s/%s)" % (self.name, self.args, self.result, self.members)
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, FunctionInstantiationSignature):
            return False
        return self.name == other.name and self.args == other.args and self.result == other.result and self.members == other.members
    
    def __hash__(self):
        return self.name.__hash__()

class ClassInstantiationSignature(object):
    def __init__(self):
        self.name = None
        self.members = []

    def __str__(self):
        return "(%s/%s/%s/%s)" % (self.name, self.members)
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ClassInstantiationSignature):
            return False
        return self.name == other.name and self.members == other.members
    
    def __hash__(self):
        return self.name.__hash__()
