
class BorrowId(object):
    def __init__(self):
        self.value = None
    
    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

    def __str__(self):
        return "($borrow.%s)" % self.value

    def __repr__(self):
        return self.__str__()

class BorrowKind(object):
    def __init__(self):
        self.local_borrow = None
        self.external_borrow = None

    def __eq__(self, other):
        return self.local_borrow == other.local_borrow and self.external_borrow == other.external_borrow

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        if self.external_borrow:
            return self.external_borrow.__hash__()
        if self.local_borrow:
            return self.local_borrow.__hash__()

    def __str__(self):
        if self.local_borrow:
            return "#local.%s" % self.local_borrow
        if self.external_borrow:
            return "#external.%s" % self.external_borrow

    def __repr__(self):
        return self.__str__()

class ExternalBorrow(object):
    def __init__(self):
        self.ownership_var = None
        self.borrow_id = None

    def __eq__(self, other):
        return self.ownership_var == other.ownership_var and self.borrow_id == other.borrow_id

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.ownership_var.__hash__()

    def __str__(self):
        return "(#external.%s/%s)" % (self.ownership_var, self.borrow_id)

    def __repr__(self):
        return self.__str__()

class BorrowMap(object):
    def __init__(self):
        self.borrows = {}

    def addLocalBorrow(self, borrowid, path):
        if borrowid not in self.borrows:
            self.borrows[borrowid] = set()
        local_borrow = BorrowKind()
        local_borrow.local_borrow = path
        self.borrows[borrowid].add(local_borrow)

    def addExternalBorrow(self, borrowid, externalid):
        if borrowid not in self.borrows:
            self.borrows[borrowid] = set()
        external_borrow = BorrowKind()
        external_borrow.external_borrow = externalid
        self.borrows[borrowid].add(external_borrow)
    
    def addKind(self, borrowid, kind):
        if borrowid not in self.borrows:
            self.borrows[borrowid] = set()
        self.borrows[borrowid].add(kind)

    def getBorrows(self, borrowid):
        return self.borrows[borrowid]