import Compiler.Ownership.TypeVariableInfo as TypeVariableInfo

class Allocator(object):
    def __init__(self):
        self._nextOwnershipVar = 0
        self._nextGroupVar = 0
        self._nextBorrow = 0

    def __str__(self):
        return "allocator(%s/%s/%s)" % (self._nextOwnershipVar, self._nextGroupVar, self._nextBorrow)

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other) -> bool:
        if not isinstance(self, Allocator):
            return False
        return self._nextOwnershipVar == other._nextOwnershipVar and self._nextGroupVar == other._nextGroupVar and self._nextBorrow == other._nextBorrow

    def nextOwnershipVar(self):
        n = self._nextOwnershipVar
        self._nextOwnershipVar += 1
        v = TypeVariableInfo.OwnershipVar()
        v.value = n
        return v

    def nextGroupVar(self):
        n = self._nextGroupVar
        self._nextGroupVar += 1
        v = TypeVariableInfo.GroupVar()
        v.value = n
        return v

    def nextBorrowVar(self):
        v = self._nextBorrow
        self._nextBorrow += 1
        return v

    def nextTypeVariableInfo(self):
        tv_info = TypeVariableInfo.TypeVariableInfo()
        tv_info.ownership_var = self.nextOwnershipVar()
        tv_info.group_var = self.nextGroupVar()
        return tv_info