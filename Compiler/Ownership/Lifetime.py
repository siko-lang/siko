import Compiler.Ownership.BorrowUtil as BorrowUtil

class Lifetime(object):
    def __init__(self, value):
        if isinstance(value, BorrowUtil.BorrowId):
            self.value = value.value
        else:
            self.value = value

    def __str__(self):
        return "'l%s" % self.value
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

def asList(lifetimes):
    ls = []
    for l in lifetimes:
        ls.append(str(l))
    return ", ".join(ls)