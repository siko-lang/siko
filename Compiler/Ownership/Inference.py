import Compiler.IR as IR
import Compiler.Util as Util
import Compiler.DependencyProcessor as DependencyProcessor
import Compiler.Ownership.BorrowUtil as BorrowUtil
import Compiler.Ownership.Path as Path

class CtorConstraint(object):
    def __init__(self):
        self.var = None

    def __str__(self):
        return "ctor %s" % (self.var)

    def __repr__(self) -> str:
        return self.__str__()

class FieldAccessConstraint(object):
    def __init__(self):
        self.root = None
        self.members = []
        self.var = None
        self.borrow = False
        self.usage = None
        self.final = None
        self.instruction_id = None

    def __str__(self):
        return "field %s.%s -> %s/%s" % (self.root, self.members, self.var, self.borrow)

    def __repr__(self) -> str:
        return self.__str__()

class Owner(object):
    def __init__(self):
        pass

    def __str__(self) -> str:
        return "owner"

    def __repr__(self) -> str:
        return self.__str__()

class Unknown(object):
    def __init__(self):
        pass

    def __str__(self) -> str:
        return "unknown"

    def __repr__(self) -> str:
        return self.__str__()

class Borrow(object):
    def __init__(self):
        self.borrow_id = None

    def __str__(self) -> str:
        return "borrow"

    def __repr__(self) -> str:
        return self.__str__()

class ConstraintHolder(object):
    def __init__(self):
        self.constraints = {}
    
    def addConstraint(self, var, constraint):
        if var not in self.constraints:
            self.constraints[var] = []
        self.constraints[var].append(constraint)

    def getConstraints(self, var):
        if var in self.constraints:
            return self.constraints[var]
        else:
            return []

    def getAll(self):
        all = []
        for (_, cs) in self.constraints.items():
            all += cs
        return all

class InferenceEngine(object):
    def __init__(self, fn, profiles, classes):
        self.fn = fn
        self.profiles = profiles
        self.classes = classes
        self.ownerships = {}
        self.borrow_map = BorrowUtil.BorrowMap()
        self.next_borrow_id = 0

    def getNextBorrowId(self):
        id = self.next_borrow_id
        self.next_borrow_id += 1
        borrow_id = BorrowUtil.BorrowId()
        borrow_id.value = id
        return borrow_id

    def infer(self):
        #print("Inference for %s" % self.fn.name)
        self.run()
        #self.dump()
        return self.ownerships

    def setOwner(self, var):
        #print("Set owner", var)
        self.ownerships[var] = Owner()

    def setBorrow(self, var, borrow_id):
        # print("Set borrow", var, borrow_id)
        b = Borrow()
        b.borrow_id = borrow_id
        self.ownerships[var] = b

    def getOwnership(self, var):
        if var in self.ownerships:
            return self.ownerships[var]
        else:
            return Unknown()

    def processFieldAccessConstraint(self, constraint):
        #print("FieldAccessConstraint %s" % constraint)
        parents = []
        for member in constraint.members:
            parents.append(member.info.ownership_var)
        parents.append(constraint.root)
        parents.reverse()
        constraint.final = Owner()
        #print("parents", parents)
        #print("ownerships", self.ownerships)
        for parent in parents:
            parent_o = self.getOwnership(parent)
            if isinstance(parent_o, Unknown):
                constraint.final = Unknown()
                break
            if isinstance(parent_o, Borrow):
                constraint.final = parent_o
                break
        #print("Final", constraint.final)
        if isinstance(constraint.final, Owner):
            if constraint.borrow:
                borrowid = self.getNextBorrowId()
                self.borrow_map.addLocalBorrow(borrowid, constraint.usage)
                (user_borrows, is_valid) = self.checkBorrows(constraint.var, borrowid)
                if is_valid:
                    borrow = Borrow()
                    borrow.borrow_id = borrowid
                    constraint.final = borrow
                    self.setBorrow(constraint.var, borrowid)
                else:
                    self.setOwner(constraint.var)
            else:
                self.setOwner(constraint.var)
        elif isinstance(constraint.final, Borrow):
            (user_borrows, is_valid) = self.checkBorrows(constraint.var, constraint.final.borrow_id)
            if is_valid:
                prev = self.getOwnership(constraint.var)
                if isinstance(prev, Borrow):
                    prev_user_borrows = self.borrow_map.getBorrows(prev.borrow_id)
                    merged = self.getNextBorrowId()
                    for b in user_borrows:
                        self.borrow_map.addKind(merged, b)
                    for b in prev_user_borrows:
                        self.borrow_map.addKind(merged, b)
                    self.setBorrow(constraint.var, merged)
                elif isinstance(prev, Unknown):
                    self.setBorrow(constraint.var, constraint.final.borrow_id)
            else:
                self.setOwner(constraint.var)

    def checkBorrows(self, target_var, borrow_id):
        # print("can %s borrow %s" % (target_var, borrow_id))
        user_borrows = self.borrow_map.getBorrows(borrow_id)
        is_valid = True
        for user_borrow in user_borrows:
            if user_borrow.external_borrow:
                pass # always valid
            if user_borrow.local_borrow:
                forbidden_borrows = self.fn.forbidden_borrows[target_var]
                # print("Borrow check??? %s %s %s" % (target_var, forbidden_borrows, user_borrow.local_borrow))
                if user_borrow.local_borrow in forbidden_borrows:
                    # print("False!")
                    is_valid = False
        return (user_borrows, is_valid)

    def processConstraints(self, groups, constraints):
        for group in groups:
            for item in group.items:
                for constraint in constraints.getConstraints(item):
                    if isinstance(constraint, CtorConstraint):
                        self.setOwner(constraint.var)
                    if isinstance(constraint, FieldAccessConstraint):
                        self.processFieldAccessConstraint(constraint)

    def collectConstraints(self):
        dep_map = {}
        constraints = ConstraintHolder()
        for arg in self.fn.ownership_signature.args:
            dep_map[arg.ownership_var] = []
        dep_map[self.fn.ownership_signature.result.ownership_var] = []
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.ValueRef):
                    for member in i.members:
                        dep_map[member.info.ownership_var] = []
                dep_map[i.tv_info.ownership_var] = []
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.DropVar):
                    # TODO FIXME
                    constraint = CtorConstraint()
                    constraint.var = i.tv_info.ownership_var
                    constraints.addConstraint(i.tv_info.ownership_var, constraint)
                elif isinstance(i, IR.BoolLiteral):
                    constraint = CtorConstraint()
                    constraint.var = i.tv_info.ownership_var
                    constraints.addConstraint(i.tv_info.ownership_var, constraint)
                elif isinstance(i, IR.NamedFunctionCall):
                    if i.ctor:
                        constraint = CtorConstraint()
                        constraint.var = i.tv_info.ownership_var
                        constraints.addConstraint(i.tv_info.ownership_var, constraint)
                    else:
                        if i.name == Util.getUnit():
                            pass # TODO
                            constraint = CtorConstraint()
                            constraint.var = i.tv_info.ownership_var
                            constraints.addConstraint(i.tv_info.ownership_var, constraint)
                        else:
                            if i.id in self.profiles:
                                profile = self.profiles[i.id]
                                for path in profile.paths:
                                    constraint = FieldAccessConstraint()
                                    constraint.borrow = False
                                    constraint.root = path.arg.ownership_var
                                    constraint.members = path.src
                                    if len(path.dest) == 0:
                                        constraint.var = path.result.ownership_var
                                    else:
                                        constraint.var = path.dest[-1].info.ownership_var
                                    constraint.instruction_id = None
                                    constraints.addConstraint(path.arg.ownership_var, constraint)
                elif isinstance(i, IR.Bind):
                    constraint = CtorConstraint()
                    constraint.var = i.tv_info.ownership_var
                    constraints.addConstraint(i.tv_info.ownership_var, constraint)
                elif isinstance(i, IR.ValueRef):
                    if i.bind_id is None:
                        root = self.fn.ownership_signature.args[i.name.value].ownership_var
                    else:
                        root_instruction = self.fn.body.getInstruction(i.bind_id)
                        root_instruction = self.fn.body.getInstruction(root_instruction.rhs)
                        root = root_instruction.tv_info.ownership_var
                    constraint = FieldAccessConstraint()
                    constraint.borrow = i.borrow
                    constraint.root = root
                    constraint.members = i.members
                    constraint.var = i.tv_info.ownership_var
                    constraint.instruction_id = i.id
                    if len(i.fields) == 0:
                        constraint.usage = Path.WholePath()
                        constraint.usage.var = i.name
                    else:
                        constraint.usage = Path.PartialPath()
                        constraint.usage.var = i.name
                        constraint.usage.fields = i.fields
                    constraints.addConstraint(i.tv_info.ownership_var, constraint)
                    for member in i.members:
                        dep_map[i.tv_info.ownership_var].append(member.info.ownership_var)
                    dep_map[i.tv_info.ownership_var].append(constraint.root)
        groups = DependencyProcessor.processDependencies(dep_map)
        return (groups, constraints)

    def setOwnerIfUnknown(self, var):
        o = self.getOwnership(var)
        if isinstance(o, Unknown):
            #print("Setting unknown to owner", var)
            self.setOwner(var)

    def unpackOwners(self, ownership_dep_map):
        def processInfo(info):
            self.setOwnerIfUnknown(info.ownership_var)
            if info.group_var in ownership_dep_map:
                vars = ownership_dep_map[info.group_var]
                for var in vars:
                    self.setOwnerIfUnknown(var)
        for arg in self.fn.ownership_signature.args:
            processInfo(arg)
        processInfo(self.fn.ownership_signature.result)
        for member in self.fn.ownership_signature.members:
            self.setOwnerIfUnknown(member.info.ownership_var)
        
    def run(self):
        (groups, constraints) = self.collectConstraints()
        #print(groups)
        # for (id, constraint) in constraints.items():
        #     print(id, constraint)
        for external_borrow in self.fn.ownership_signature.borrows:
            borrow_id = self.getNextBorrowId()
            self.borrow_map.addExternalBorrow(borrow_id, external_borrow.borrow_id)
            self.setBorrow(external_borrow.ownership_var, borrow_id)
        for var in self.fn.ownership_signature.owners:
            self.setOwner(var)
        self.processConstraints(groups, constraints)
        #self.dump();
        for c in constraints.getAll():
            if isinstance(c, FieldAccessConstraint):
                if c.instruction_id == None:
                    continue
                i = self.fn.body.getInstruction(c.instruction_id)
                res_o = self.getOwnership(i.tv_info.ownership_var)
                #print("final %s, res %s, %s, %s, borrow:%s" % (c.final, res_o, i.tv_info, c.instruction_id, i.borrow))
                if isinstance(c.final,  Owner) and isinstance(res_o, Owner):
                    if i.borrow:
                        i.clone = True
                if isinstance(c.final, Borrow) and isinstance(res_o, Owner):
                    i.clone = True
                if i.clone:
                    clazz = self.classes[i.type.value]
                    if "Clone" not in clazz.derives:
                        # self.dump()
                        Util.error("Cannot be cloned! %s at %s" % (i.type, i))

    def dump(self):
        print("forbidden", self.fn.forbidden_borrows)
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                member_ownerships = {}
                members = i.members
                for member in i.members:
                    member_ownerships[member.info.ownership_var] = self.getOwnership(member.info.ownership_var)
                ownership = self.getOwnership(i.tv_info.ownership_var)
                borrows = []
                if isinstance(ownership, Borrow):
                    borrows = self.borrow_map.getBorrows(ownership.borrow_id)
                print("%5s %35s %10s %s %s %s %s" % (i.id, i, i.tv_info, ownership, members, member_ownerships, borrows))
