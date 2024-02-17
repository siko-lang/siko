import IR
import Util
import DependencyProcessor

class ConverterConstraint(object):
    def __init__(self):
        self.from_var = None
        self.to_var = None

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
        pass

    def __str__(self) -> str:
        return "borrow"

    def __repr__(self) -> str:
        return self.__str__()

class InferenceEngine(object):
    def __init__(self):
        self.fn = None
        self.ownerships = {}

    def inferFn(self, fn):
        self.fn = fn
        print("Inference for %s" % fn.name)
        self.initialize()
        self.dump()

    def setOwner(self, var):
        self.ownerships[var] = Owner()

    def getOwnership(self, var):
        if var in self.ownerships:
            return self.ownerships[var]
        else:
            return Unknown()

    def initialize(self):
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
                    constraint = FieldAccessConstraint()
                    constraint.root = root_instruction.tv_info.ownership_var
                    constraint.members = i.members
                    constraint.var = i.tv_info.ownership_var
                    constraints[i.tv_info.ownership_var] = constraint
                    for member in i.members:
                        dep_map[i.tv_info.ownership_var].append(member.info.ownership_var)
        groups = DependencyProcessor.processDependencies(dep_map)
        print(groups)
        for (id, constraint) in constraints.items():
            print(id, constraint)
        for group in groups:
            for item in group.items:
                print("Checking %s", item)
                if item not in constraints:
                    print("No constraints??")
                else:
                    constraint = constraints[item]
                    if isinstance(constraint, CtorConstraint):
                        self.setOwner(constraint.var)
                    if isinstance(constraint, ConverterConstraint):
                        from_o = self.getOwnership(constraint.from_var)
                        if isinstance(from_o, Owner):
                            self.setOwner(constraint.to_var)
                    if isinstance(constraint, FieldAccessConstraint):
                        pass

    def dump(self):
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                members = []
                member_ownerships = {}
                if isinstance(i, IR.ValueRef):
                    members = i.members
                    for member in i.members:
                        member_ownerships[member.info.ownership_var] = self.getOwnership(member.info.ownership_var)
                ownership = self.getOwnership(i.tv_info.ownership_var)
                print("%5s %35s %10s %s %s %s" % (i.id, i, i.tv_info, ownership, members, member_ownerships))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)