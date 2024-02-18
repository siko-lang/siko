import IR
import Util
import DependencyProcessor
import BorrowUtil

class ConverterConstraint(object):
    def __init__(self):
        self.from_var = None
        self.to_var = None
        self.source_id = None

    def __str__(self):
        return "converter %s -> %s" % (self.from_var, self.to_var)

    def __repr__(self) -> str:
        return self.__str__()

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

    def __str__(self):
        return "field %s.%s -> %s" % (self.root, self.members, self.var)

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

class InferenceEngine(object):
    def __init__(self):
        self.fn = None
        self.classes = None
        self.ownerships = {}
        self.borrow_map = BorrowUtil.BorrowMap()

    def inferFn(self, fn, classes):
        self.fn = fn
        self.classes = classes
        #print("Inference for %s" % fn.name)
        self.run()
        #self.dump()

    def setOwner(self, var):
        #print("Set owner", var)
        self.ownerships[var] = Owner()

    def setBorrow(self, var, borrow_id):
        #print("Set borrow", var)
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
        final = Owner()
        #print("parents", parents)
        #print("ownerships", self.ownerships)
        for parent in parents:
            parent_o = self.getOwnership(parent)
            if isinstance(parent_o, Unknown):
                final = Unknown()
                break
            if isinstance(parent_o, Borrow):
                final = Borrow()
                break
        #print("Final", final)
        if isinstance(final, Owner):
            self.setOwner(constraint.var)
        elif isinstance(final, Borrow):
            self.setBorrow(constraint.var)

    def checkBorrows(self, target_var, borrow_id):
        user_borrows = self.borrow_map.getBorrows(borrow_id)
        is_valid = True
        for user_borrow in user_borrows:
            if user_borrow.external_borrow:
                pass # always valid
            if user_borrow.local_borrow:
                forbidden_borrows = self.fn.forbidden_borrows[target_var]
                #print("Borrow %s %s %s" % (constraint.to_var, forbidden_borrows, arg.usage))
                if user_borrow.local_borrow in forbidden_borrows:
                    is_valid = False
        return (user_borrows, is_valid)

    def processConverterConstraint(self, constraint):
        from_o = self.getOwnership(constraint.from_var)
        to_o = self.getOwnership(constraint.to_var)
        if isinstance(from_o, Owner) and isinstance(to_o, Unknown):
            self.setOwner(constraint.to_var)
        if isinstance(from_o, Owner) and isinstance(to_o, Borrow):
            pass # TODO
        if isinstance(from_o, Owner) and isinstance(to_o, Owner):
            pass # nothing to do
        if isinstance(from_o, Borrow) and isinstance(to_o, Unknown):
            (user_borrows, is_valid) = self.checkBorrows(constraint.to_var, from_o.borrow_id)
            if is_valid:
                self.setBorrow(constraint.to_var, from_o.borrow_id)
            else:
                self.setOwner(constraint.to_var)
        if isinstance(from_o, Borrow) and isinstance(to_o, Borrow):
            pass # TODO
        if isinstance(from_o, Borrow) and isinstance(to_o, Owner):
            pass # TODO

    def processConstraints(self, groups, constraints):
        for group in groups:
            for item in group.items:
                if item in constraints:
                    constraint = constraints[item]
                    if isinstance(constraint, CtorConstraint):
                        self.setOwner(constraint.var)
                    if isinstance(constraint, ConverterConstraint):
                        self.processConverterConstraint(constraint)
                    if isinstance(constraint, FieldAccessConstraint):
                        self.processFieldAccessConstraint(constraint)

    def collectConstraints(self):
        dep_map = {}
        constraints = {}
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.ValueRef):
                    for member in i.members:
                        dep_map[member.info.ownership_var] = []
                dep_map[i.tv_info.ownership_var] = []
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.NamedFunctionCall):
                    if i.ctor:
                        constraint = CtorConstraint()
                        constraint.var = i.tv_info.ownership_var
                        constraints[i.tv_info.ownership_var] = constraint
                elif isinstance(i, IR.Converter):
                    arg = self.fn.body.getInstruction(i.arg)
                    constraint = ConverterConstraint()
                    constraint.source_id = i.arg
                    constraint.from_var = arg.tv_info.ownership_var
                    constraint.to_var = i.tv_info.ownership_var
                    constraints[i.tv_info.ownership_var] = constraint
                    dep_map[i.tv_info.ownership_var] = [arg.tv_info.ownership_var]
                elif isinstance(i, IR.Bind):
                    constraint = CtorConstraint()
                    constraint.var = i.tv_info.ownership_var
                    constraints[i.tv_info.ownership_var] = constraint
                elif isinstance(i, IR.ValueRef):
                    root_instruction = self.fn.body.getInstruction(i.bind_id)
                    root_instruction = self.fn.body.getInstruction(root_instruction.rhs)
                    constraint = FieldAccessConstraint()
                    constraint.root = root_instruction.tv_info.ownership_var
                    constraint.members = i.members
                    constraint.var = i.tv_info.ownership_var
                    constraints[i.tv_info.ownership_var] = constraint
                    for member in i.members:
                        dep_map[i.tv_info.ownership_var].append(member.info.ownership_var)
                    dep_map[i.tv_info.ownership_var].append(constraint.root)
        groups = DependencyProcessor.processDependencies(dep_map)
        return (groups, constraints)

    def run(self):
        #print(groups)
        #for (id, constraint) in constraints.items():
            #print(id, constraint)
        (groups, constraints) = self.collectConstraints()
        self.processConstraints(groups, constraints)
        for block in self.fn.body.blocks:
            for (index, i) in enumerate(block.instructions):
                if isinstance(i, IR.Converter):
                    arg = self.fn.body.getInstruction(i.arg)
                    arg_o = self.ownerships[arg.tv_info.ownership_var]
                    res_o = self.ownerships[i.tv_info.ownership_var]
                    if isinstance(arg_o,  Owner) and isinstance(res_o, Owner):
                        if arg.borrow:
                            clazz = self.classes[arg.type.value]
                            if "Clone" not in clazz.derives:
                                Util.error("Cannot be cloned! %s" % arg.type)
                            clone = IR.Clone()
                            clone.id = i.id
                            clone.arg = i.arg
                            block.instructions[index] = clone
                        else:
                            move = IR.Move()
                            move.id = i.id
                            move.arg = i.arg
                            block.instructions[index] = move

    def dump(self):
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                member_ownerships = {}
                members = i.members
                for member in i.members:
                    member_ownerships[member.info.ownership_var] = self.getOwnership(member.info.ownership_var)
                ownership = self.getOwnership(i.tv_info.ownership_var)
                print("%5s %35s %10s %s %s %s" % (i.id, i, i.tv_info, ownership, members, member_ownerships))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f, program.classes)