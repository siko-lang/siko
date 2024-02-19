
class Substitution(object):
    def __init__(self):
        self.ownership_vars = {}
        self.group_vars = {}

    def addOwnershipVar(self, ownership_var, other):
        if ownership_var != other:
            self.ownership_vars[ownership_var] = other

    def addGroupVar(self, group_var, other):
        if group_var != other:
            self.group_vars[group_var] = other

    def applyOwnershipVar(self, var):
        res = var
        while True:
            if res in self.ownership_vars:
                res = self.ownership_vars[res]
            else:
                return res

    def applyGroupVar(self, var):
        res = var
        while True:
            if res in self.group_vars:
                res = self.group_vars[res]
            else:
                return res

    def applyTypeVariableInfo(self, info):
        res = TypeVariableInfo()
        res.ownership_var = self.applyOwnershipVar(info.ownership_var)
        res.group_var = self.applyGroupVar(info.group_var)
        return res

class OwnershipVar(object):
    def __init__(self):
        self.value = 0

    def __str__(self):
        return "%%%s" % self.value

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

class GroupVar(object):
    def __init__(self):
        self.value = 0

    def __str__(self):
        return "#%s" % self.value
    
    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

class TypeVariableInfo(object):
    def __init__(self):
        self.ownership_var = None
        self.group_var = None

    def __str__(self):
        return "(%s, %s)" % (self.ownership_var, self.group_var)

    def __eq__(self, other):
        return self.ownership_var == other.ownership_var and self.group_var == other.group_var

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.ownership_var.__hash__()
    
    def __repr__(self) -> str:
        return self.__str__()