import Compiler.Ownership.Allocator as Allocator

class ExternalBorrow(object):
    def __init__(self) -> None:
        self.ownership_var = None
        self.borrow_id = None

    def __str__(self):
        return "(%s/%s)" % (self.ownership_var, self.borrow_id)
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ExternalBorrow):
            return False
        return self.ownership_var == other.ownership_var and self.borrow_id == other.borrow_id
    
    def __hash__(self):
        return self.ownership_var.__hash__()

class FunctionOwnershipSignature(object):
    def __init__(self):
        self.name = None
        self.args = []
        self.result = None
        self.members = []
        self.borrows = []
        self.allocator = Allocator.Allocator()

    def __str__(self):
        return "(%s/%s/%s/%s/%s/%s)" % (self.name, self.args, self.result, self.members, self.borrows, self.allocator)
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, FunctionOwnershipSignature):
            return False
        return self.name == other.name and self.args == other.args and self.result == other.result and \
            self.members == other.members and self.borrows == other.borrows and self.allocator == other.allocator
    
    def __hash__(self):
        return self.name.__hash__()

class ClassInstantiationSignature(object):
    def __init__(self):
        self.name = None
        self.root = None
        self.members = []
        self.borrows = []

    def __str__(self):
        return "(%s/%s/%s/%s)" % (self.name, self.root, self.members, self.borrows)
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ClassInstantiationSignature):
            return False
        return self.name == other.name and self.root == other.root and self.members == other.members and self.borrows == other.borrows
    
    def __hash__(self):
        return self.name.__hash__()
