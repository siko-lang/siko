class WholePath(object):
    def __init__(self, isDrop = False):
        self.var = None
        self.is_drop = isDrop

    def __str__(self):
        return "whole(%s)" % (self.var)

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other):
        if isinstance(other, WholePath):
            return self.var == other.var
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.var.__hash__()

class PartialPath(object):
    def __init__(self):
        self.var = None
        self.fields = []

    def __str__(self):
        fields = ".".join(self.fields)
        return "partial(%s.%s)" % (self.var, fields)
    
    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other):
        if isinstance(other, PartialPath):
            if self.var != other.var:
                return False
            if len(self.fields) != len(other.fields):
                return False
            for (index, v) in enumerate(self.fields):
                if v != other.fields[index]:
                    return False
            return True
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.var.__hash__()
