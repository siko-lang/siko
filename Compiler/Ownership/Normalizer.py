import Compiler.Ownership.Signatures as Signatures
import Compiler.Ownership.Inference as Inference
import Compiler.Ownership.MemberInfo as MemberInfo
import Compiler.Ownership.BorrowUtil as BorrrowUtil
import Compiler.Ownership.Allocator as Allocator
import Compiler.Ownership.TypeVariableInfo as TypeVariableInfo
import copy

class OwnershipProvider(object):
    def __init__(self):
        self.ownership_map = None
        self.borrow_list = None

    def getBorrow(self, var):
        if self.ownership_map is not None:
            if var in self.ownership_map:
                o = self.ownership_map[var]
                if isinstance(o, Inference.Borrow):
                    return o.borrow_id
        if self.borrow_list is not None:
            for b in self.borrow_list:
                if b.ownership_var == var:
                    return b.borrow_id
        return None

    def isOwner(self, var):
        if self.ownership_map is not None:
            if var in self.ownership_map:
                o = self.ownership_map[var]
                return isinstance(o, Inference.Owner)
        return False

class Normalizer(object):
    def __init__(self):
        self.allocator = Allocator.Allocator()
        self.ownership_vars = {}
        self.group_vars = {}
        self.borrows = {}

    def normalizeOwnershipVar(self, var):
        if var in self.ownership_vars:
            ownership_var = self.ownership_vars[var]
        else:
            ownership_var = self.allocator.nextOwnershipVar()
            self.ownership_vars[var] = ownership_var
        return ownership_var
        
    def normalizeGroupVar(self, var):
        if var in self.group_vars:
            group_var = self.group_vars[var]
        else:
            group_var = self.allocator.nextGroupVar()
            self.group_vars[var] = group_var
        return group_var

    def normalizeBorrow(self, borrow):
        if borrow in self.borrows:
            b = self.borrows[borrow]
        else:
            b = copy.deepcopy(borrow)
            b.value = self.allocator.nextBorrowVar()
            self.borrows[borrow] = b
        return b

    def normalize(self, info):
        res = copy.deepcopy(info)
        res.ownership_var = self.normalizeOwnershipVar(info.ownership_var)
        res.group_var = self.normalizeGroupVar(info.group_var)
        return res

def filterOutMembers(groups, ownership_dep_map, members, ownership_provider, borrows, owners, onlyBorrow):
    #print("groups", groups)
    #print("ownership_dep_map", ownership_dep_map)
    #print("members", members)
    relevant_members = []
    for group_var in groups:
        if group_var in ownership_dep_map:
            ownership_vars = ownership_dep_map[group_var]
            #print("Arg", arg)
            #print("vars", ownership_vars)
            for member in members:
                if member.info.ownership_var in ownership_vars:
                    #print("member is relevant", member)
                    relevant_members.append(member)
    #print("relevant_members", relevant_members)
    for member in relevant_members:
        if ownership_provider.getBorrow(member.info.ownership_var) is not None:
            borrows.append(member.info.ownership_var)
        if ownership_provider.isOwner(member.info.ownership_var):
            owners.append(member.info.ownership_var)
    #print("Borrows", borrows)
    if onlyBorrow:
        relevant_vars = borrows
    else:
        relevant_vars = borrows + owners
    filtered_members = []
    for member in relevant_members:
        ownership_vars = ownership_dep_map[member.info.group_var]
        containsRelevant = member.info.ownership_var in relevant_vars
        for v in ownership_vars:
            if v in relevant_vars:
                containsRelevant = True
                break
        if containsRelevant:
            filtered_members.append(member)
    #print("filtered_members", filtered_members)
    return (filtered_members, borrows, owners)

def collectChildMembers(normalizer, var, members):
    def sortFunc(member):
        return member.kind.index
    children = []
    for member in members:
        if member.root == var:
            children.append(member)
    children.sort(key=sortFunc)
    normalized_children = []
    for child in children:
        normalized_child = copy.deepcopy(child)
        normalized_child.root = normalizer.normalizeGroupVar(child.root)
        normalized_child.info = normalizer.normalize(child.info)
        if normalized_child not in normalized_children:
            normalized_children.append(normalized_child)
    for child in children:
        sub = collectChildMembers(normalizer, child.info.group_var, members)
        for m in sub:
            if m not in normalized_children:
                normalized_children.append(m)
    return normalized_children

def normalizeFunctionOwnershipSignature(signature, ownership_dep_map, members, ownership_provider, onlyBorrow):
    normalizer = Normalizer()
    #print("Signature", signature)
    #print("ownership_dep_map", ownership_dep_map)
    #print("members", members)
    groups = []
    borrows = []
    owners = []
    for arg in signature.args:
        groups.append(arg.group_var)
        if ownership_provider.getBorrow(arg.ownership_var) is not None:
            borrows.append(arg.ownership_var)
        if ownership_provider.isOwner(arg.ownership_var):
            owners.append(arg.ownership_var)
    if ownership_provider.getBorrow(signature.result.ownership_var) is not None:
        borrows.append(signature.result.ownership_var)
    if ownership_provider.isOwner(signature.result.ownership_var):
        owners.append(signature.result.ownership_var)
    (filtered_members, borrows, owners) = filterOutMembers(groups, ownership_dep_map, members,
                                                           ownership_provider, borrows, owners, onlyBorrow)
    ordered_members = []
    normalized_args = []
    for arg in signature.args:
        normalized_args.append(normalizer.normalize(arg))
    normalized_result = normalizer.normalize(signature.result)
    for arg in signature.args:
        sub = collectChildMembers(normalizer, arg.group_var, filtered_members)
        for m in sub:
            if m not in ordered_members:
                ordered_members.append(m)
    #print("Ordered members", ordered_members)
    normalized_borrows = []
    for borrower in borrows:
        borrow_id = ownership_provider.getBorrow(borrower)
        normalized_borrow = BorrrowUtil.ExternalBorrow()
        normalized_borrow.borrow_id = normalizer.normalizeBorrow(borrow_id)
        normalized_borrow.ownership_var = normalizer.normalizeOwnershipVar(borrower)
        if normalized_borrow not in normalized_borrows:
            normalized_borrows.append(normalized_borrow)
    normalized_owners = []
    if not onlyBorrow:
        for v in owners:
            normalized_owners.append(normalizer.normalizeOwnershipVar(v))
    signature.args = normalized_args
    signature.members = ordered_members
    signature.result = normalized_result
    signature.allocator = normalizer.allocator
    signature.borrows = normalized_borrows
    signature.owners = normalized_owners
    #print("Signature2", signature)
    return signature

def normalizeClassOwnershipSignature(signature, info, ownership_dep_map, members, ownership_provider):
    normalizer = Normalizer()
    #print("Signature", signature)
    #print("ownership_dep_map", ownership_dep_map)
    #print("members", members)
    borrows = []
    owners = []
    (filtered_members, borrows, owners) = filterOutMembers([info.group_var], ownership_dep_map, members, ownership_provider, borrows, owners, onlyBorrow=True)
    normalized_root = normalizer.normalize(info)
    ordered_members = collectChildMembers(normalizer, info.group_var, filtered_members)
    #print("Ordered members", ordered_members)
    normalized_borrows = []
    for borrower in borrows:
        borrow_id = ownership_provider.getBorrow(borrower)
        normalized_borrow = BorrrowUtil.ExternalBorrow()
        normalized_borrow.borrow_id = normalizer.normalizeBorrow(borrow_id)
        normalized_borrow.ownership_var = normalizer.normalizeOwnershipVar(borrower)
        if normalized_borrow not in normalized_borrows:
            normalized_borrows.append(normalized_borrow)
    signature.root = normalized_root
    signature.members = ordered_members
    signature.allocator = normalizer.allocator
    signature.borrows = normalized_borrows
    #print("Signature2", signature)
    return signature
