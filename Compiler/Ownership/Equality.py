import Compiler.IR as IR
import Compiler.Util as Util
import Compiler.Ownership.Signatures as Signatures
import Compiler.Ownership.TypeVariableInfo as TypeVariableInfo
import Compiler.Ownership.MemberInfo as MemberInfo
import Compiler.Ownership.Instantiator as Instantiator
import copy

class EqualityEngine(object):
    def __init__(self, fn, profile_store):
        self.fn = fn
        self.tv_info_vars = {}
        self.substitution = TypeVariableInfo.Substitution()
        self.profile_store = profile_store
        self.profiles = []

    def process(self):
        #print("Equality for %s/%s" % (self.fn.name, self.fn.ownership_signature))
        self.initialize()
        self.mergeInstructions()
        self.mergeMembers()
        self.finalize()
        #self.dump()

    def nextOwnershipVar(self):
        return self.fn.ownership_signature.allocator.nextOwnershipVar()

    def nextGroupVar(self):
        return self.fn.ownership_signature.allocator.nextGroupVar()

    def nextTypeVariableInfo(self):
        return self.fn.ownership_signature.allocator.nextTypeVariableInfo()

    def initialize(self):
        if self.fn.ownership_signature is None:
            self.fn.ownership_signature = Signatures.FunctionOwnershipSignature()
            self.fn.ownership_signature.result = self.nextTypeVariableInfo()
            for arg in self.fn.args:
                self.fn.ownership_signature.args.append(self.nextTypeVariableInfo())
        for (index, arg) in enumerate(self.fn.args):
            self.tv_info_vars[arg.name] = self.fn.ownership_signature.args[index]
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
                        member_info.kind.type = MemberInfo.FieldKind
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
                            member_info.kind.type = MemberInfo.FieldKind
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
                else:
                    if i.name == Util.getUnit():
                        pass # TODO, remove this
                    else:
                        profile = self.profile_store.getProfile(i.name)
                        profile = copy.deepcopy(profile)
                        (signature, allocator) = Instantiator.instantiateFunctionOwnershipSignature(profile.signature, self.fn.ownership_signature.allocator)
                        self.fn.ownership_signature.allocator = allocator
                        profile.signature = signature
                        self.profiles.append(profile)
                        for (index, arg) in enumerate(i.args):
                            sig_arg_info = profile.signature.args[index]
                            arg_info = self.getInstructionTypeVariableInfo(arg)
                            self.unify(arg_info, sig_arg_info)
                        res_info = self.getInstructionTypeVariableInfo(i.id)
                        self.unify(res_info, profile.signature.result)
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
                self.unifyInstrs(t_id, i.id)
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.BlockRef):
                b = self.fn.body.getBlock(i.value)
                self.processBlock(b)
                l_id = b.getLastReal().id
                self.unifyInstrs(i.id, l_id)
            elif isinstance(i, IR.Nop):
                pass
            else:
                Util.error("OI: grouping not handling %s %s" % (type(i), i))

    def mergeInstructions(self):
        block = self.fn.body.getFirst()
        self.processBlock(block)
        ret = self.getInstructionTypeVariableInfo(block.getLastReal().id)
        self.unify(self.fn.ownership_signature.result, ret)
    
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

    def applySignature(self, sig):
        args = []
        for arg in sig.args:
            args.append(self.substitution.applyTypeVariableInfo(arg))
        owners = []
        for owner in sig.owners:
            owners.append(self.substitution.applyOwnershipVar(owner))
        for member in sig.members:
            member.root = self.substitution.applyGroupVar(member.root)
            member.info = self.substitution.applyTypeVariableInfo(member.info)
        sig.args = args
        sig.owners = owners
        sig.result = self.substitution.applyTypeVariableInfo(sig.result)

    def finalize(self):
        for block in self.fn.body.blocks:
            for i in block.instructions:
                for member in i.members:
                    member.root = self.substitution.applyGroupVar(member.root)
                    member.info = self.substitution.applyTypeVariableInfo(member.info)
                i.tv_info = self.substitution.applyTypeVariableInfo(i.tv_info)
        self.applySignature(self.fn.ownership_signature)
        for profile in self.profiles:
            self.applySignature(profile.signature)
        for profile in self.profiles:
            for owner in profile.signature.owners:
                self.fn.ownership_signature.owners.append(owner)
    
    def dump(self):
        print("Sig:", self.fn.ownership_signature)
        for profile in self.profiles:
            print("Profile sig:", profile.signature)
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                print("%5s %35s - %4s %s" % (i.id, i, i.tv_info, i.members))
