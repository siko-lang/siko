import IR
import Util
import TypeVariableInfo
import MemberInfo

class InferenceEngine(object):
    def __init__(self):
        self.fn = None
        self.next = 0
        self.tv_info_vars = {}
        self.substitution = TypeVariableInfo.Substitution()

    def inferFn(self, fn):
        self.fn = fn
        #print("Equality for %s" % fn.name)
        self.initialize()
        self.mergeInstructions()
        self.mergeMembers()
        self.finalize()
        #self.dump()

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

    def initialize(self):
        for arg in self.fn.args:
            tv_info = self.nextTypeVariableInfo()
            self.tv_info_vars[arg.name] = tv_info
        for block in self.fn.body.blocks:
            for i in block.instructions:
                i.tv_info = self.nextTypeVariableInfo()
                if isinstance(i, IR.Bind):
                    tv_info = self.nextTypeVariableInfo()
                    self.tv_info_vars[i.name] = tv_info
                if isinstance(i, IR.ValueRef):
                    root = self.nextGroupVar()
                    for index in i.indices:
                        member_info = MemberInfo.MemberInfo()
                        member_info.root = root
                        member_info.kind = MemberInfo.MemberKind()
                        member_info.kind.type = "field"
                        member_info.kind.index = index
                        member_info.info = self.nextTypeVariableInfo()
                        root = member_info.info.group_var
                        i.members.append(member_info)
                    if len(i.members) != 0:
                        i.members[-1].info.group_var = i.tv_info.group_var
                if isinstance(i, IR.NamedFunctionCall):
                    if i.ctor:
                        for (index, arg) in enumerate(i.args):
                            member_info = MemberInfo.MemberInfo()
                            member_info.root = i.tv_info.group_var
                            member_info.kind = MemberInfo.MemberKind()
                            member_info.kind.type = "field"
                            member_info.kind.index = index
                            member_info.info = self.nextTypeVariableInfo()
                            i.members.append(member_info)

    def unifyOwnership(self, o1, o2):
        o1 = self.substitution.applyOwnershipVar(o1)
        o2 = self.substitution.applyOwnershipVar(o2)
        self.substitution.addOwnershipVar(o1, o2)

    def unifyGroup(self, g1, g2):
        g1 = self.substitution.applyGroupVar(g1)
        g2 = self.substitution.applyGroupVar(g2)
        self.substitution.addGroupVar(g1, g2)

    def unify(self, info1, info2):
        self.unifyOwnership(info1.ownership_var, info2.ownership_var)
        self.unifyGroup(info1.group_var, info2.group_var)

    def unifyInstrAndVar(self, id, name):
        info1 = self.getInstructionTypeVariableInfo(id)
        info2 = self.tv_info_vars[name]
        self.unify(info1, info2)

    def unifyInstrs(self, id1, id2):
        info1 = self.getInstructionTypeVariableInfo(id1)
        info2 = self.getInstructionTypeVariableInfo(id2)
        self.unify(info1, info2)

    def getInstructionTypeVariableInfo(self, id):
        return self.fn.body.getInstruction(id).tv_info

    def processBlock(self, block):
        for i in block.instructions:
            if isinstance(i, IR.Bind):
                self.unifyInstrAndVar(i.rhs, i.name)
            elif isinstance(i, IR.NamedFunctionCall):
                if i.ctor:
                    for (index, arg) in enumerate(i.args):
                        member_info = i.members[index]
                        arg_info = self.getInstructionTypeVariableInfo(arg)
                        self.unify(arg_info, member_info.info)
            elif isinstance(i, IR.ValueRef):
                if len(i.members) == 0:
                    self.unifyGroup(self.tv_info_vars[i.name].group_var, i.tv_info.group_var)    
                else:
                    self.unifyGroup(self.tv_info_vars[i.name].group_var, i.members[0].root)
            elif isinstance(i, IR.BoolLiteral):
                pass
            elif isinstance(i, IR.If):
                true_branch = self.fn.body.getBlock(i.true_branch)
                false_branch = self.fn.body.getBlock(i.false_branch)
                self.processBlock(true_branch)
                self.processBlock(false_branch)
                t_id = true_branch.getLastReal().id
                f_id = false_branch.getLastReal().id
                self.unifyInstrs(t_id, f_id)
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.BlockRef):
                b = self.fn.body.getBlock(i.value)
                self.processBlock(b)
                l_id = b.getLastReal().id
                self.unifyInstrs(i.id, l_id)
            elif isinstance(i, IR.Converter):
                info1 = self.getInstructionTypeVariableInfo(i.id)
                info2 = self.getInstructionTypeVariableInfo(i.arg)
                self.unifyGroup(info1.group_var, info2.group_var)
            elif isinstance(i, IR.Nop):
                pass
            else:
                Util.error("OI: grouping not handling %s %s" % (type(i), i))

    def mergeInstructions(self):
        block = self.fn.body.getFirst()
        self.processBlock(block)
    
    def mergeMembers(self):
        while True:
            member_map = {}
            for block in self.fn.body.blocks:
                for i in block.instructions:
                    for member in i.members:
                        member.root = self.substitution.applyGroupVar(member.root)
                        member_map[(member.root, member.kind.index)] = []
            
            for block in self.fn.body.blocks:
                for i in block.instructions:
                    for member in i.members:
                        member.info = self.substitution.applyTypeVariableInfo(member.info)
                        member_map[(member.root, member.kind.index)].append(member.info)
        
            unified = False
            for entries in member_map.values():
                entries = list(set(entries))
                if len(entries) > 1:
                    first = entries[0]
                    first = self.substitution.applyTypeVariableInfo(first)
                    for entry in entries:
                        entry = self.substitution.applyTypeVariableInfo(entry)
                        self.unify(first, entry)
                        unified = True
            if not unified:
                break

    def finalize(self):
        for block in self.fn.body.blocks:
            for i in block.instructions:
                for member in i.members:
                    member.root = self.substitution.applyGroupVar(member.root)
                    member.info = self.substitution.applyTypeVariableInfo(member.info)
                i.tv_info = self.substitution.applyTypeVariableInfo(i.tv_info)
    
    def dump(self):
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                print("%5s %35s - %4s %s" % (i.id, i, i.tv_info, i.members))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)