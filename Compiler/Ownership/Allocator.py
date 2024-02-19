import Compiler.Ownership.TypeVariableInfo as TypeVariableInfo

class Allocator(object):
    def __init__(self):
        self.next = 0

    def __str__(self):
        return "allocator(%s)" % self.next

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        if not isinstance(self, Allocator):
            return False
        return self.next == other.next

    def nextOwnershipVar(self):
        n = self.next
        self.next += 1
        v = TypeVariableInfo.OwnershipVar()
        v.value = n
        return v

    def nextGroupVar(self):
        n = self.next
        self.next += 1
        v = TypeVariableInfo.GroupVar()
        v.value = n
        return v

    def nextTypeVariableInfo(self):
        tv_info = TypeVariableInfo.TypeVariableInfo()
        tv_info.ownership_var = self.nextOwnershipVar()
        tv_info.group_var = self.nextGroupVar()
        return tv_info