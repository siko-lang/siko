import copy

class Instantiator(object):
    def __init__(self, allocator):
        self.allocator = allocator
        self.ownership_vars = {}
        self.group_vars = {}
        self.borrows = {}

    def instantiateOwnershipVar(self, var):
        if var in self.ownership_vars:
            ownership_var = self.ownership_vars[var]
        else:
            ownership_var = self.allocator.nextOwnershipVar()
            self.ownership_vars[var] = ownership_var
        return ownership_var
        
    def instantiateGroupVar(self, var):
        if var in self.group_vars:
            group_var = self.group_vars[var]
        else:
            group_var = self.allocator.nextGroupVar()
            self.group_vars[var] = group_var
        return group_var

    def instantiateBorrow(self, borrow):
        if borrow in self.borrows:
            b = self.borrows[borrow]
        else:
            b = copy.deepcopy(borrow)
            b.value = self.allocator.nextBorrowVar()
            self.borrows[borrow] = b
        return b

    def instantiate(self, info):
        res = copy.deepcopy(info)
        res.ownership_var = self.instantiateOwnershipVar(info.ownership_var)
        res.group_var = self.instantiateGroupVar(info.group_var)
        return res

def instantiateFunctionOwnershipSignature(signature, allocator):
    instantiator = Instantiator(allocator)
    args = []
    for arg in signature.args:
        args.append(instantiator.instantiate(arg))
    result = instantiator.instantiate(signature.result)
    members = []
    for member in signature.members:
        member.root = instantiator.instantiateGroupVar(member.root)
        member.info = instantiator.instantiate(member.info)
        members.append(member)
    borrows = []
    for borrow in signature.borrows:
        borrow.ownership_var = instantiator.instantiateOwnershipVar(borrow.ownership_var)
        borrows.append(borrow)
    owners = []
    for owner in signature.owners:
        owner = instantiator.instantiateOwnershipVar(owner)
        owners.append(owner)
    signature.args = args
    signature.members = members
    signature.result = result
    signature.allocator = None
    signature.borrows = borrows
    signature.owners = owners
    return (signature, instantiator.allocator)
