import Compiler.Ownership.Signatures as Signatures
import Compiler.Ownership.TypeVariableInfo as TypeVariableInfo
import Compiler.Util as Util
import Compiler.IR as IR
import Compiler.Ownership.Inference as Inference
import Compiler.Ownership.Equality as Equality
import Compiler.Ownership.ForbiddenBorrows as ForbiddenBorrows
import Compiler.Ownership.MemberInfo as MemberInfo
import Compiler.Ownership.Normalizer as Normalizer
import copy

class Monomorphizer(object):
    def __init__(self, program, profile_store):
        self.program = program
        self.functions = {}
        self.classes = {}
        self.queue = []
        self.profile_store = profile_store

    def addClass(self, signature):
        self.queue.append(signature)

    def addFunction(self, signature):
        self.queue.append(signature)

    def processFunction(self, signature):
        if signature not in self.functions:
            if signature.name == Util.getUnit():
                return
            print("Processing fn %s" % signature)
            fn = self.program.functions[signature.name]
            fn = copy.deepcopy(fn)
            fn.ownership_signature = copy.deepcopy(signature)
            self.functions[signature] = fn
            equality = Equality.EqualityEngine(fn, self.profile_store)
            equality.process()
            members = fn.getAllMembers()
            #print("members", members)
            ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
            forbidden_borrows = ForbiddenBorrows.ForbiddenBorrowsEngine()
            #print("ownership_dep_map", ownership_dep_map)
            forbidden_borrows.process(fn, ownership_dep_map)
            inference = Inference.InferenceEngine(fn, self.profile_store, self.program.classes)
            inference.unpackOwners(ownership_dep_map)
            ownerships = inference.infer()
            ownership_provider = Normalizer.OwnershipProvider()
            ownership_provider.ownership_map = ownerships
            #print("ownerships", ownerships)
            for (index, arg) in enumerate(fn.args):
                arg_tv_info = signature.args[index]
                tsignature = Signatures.ClassInstantiationSignature()
                tsignature.name = arg.type
                tsignature = Normalizer.normalizeClassOwnershipSignature(tsignature, 
                                                                         arg_tv_info,
                                                                         ownership_dep_map,
                                                                         members,
                                                                         ownership_provider)
                arg.type = tsignature
                arg.ownership = ownerships[arg_tv_info.ownership_var]
                self.addClass(tsignature)
            rsignature = Signatures.ClassInstantiationSignature()
            rsignature.name = fn.return_type
            ret_tv_info = copy.deepcopy(signature.result)
            rsignature = Normalizer.normalizeClassOwnershipSignature(rsignature, 
                                                                     ret_tv_info,
                                                                     ownership_dep_map,
                                                                     members,
                                                                     ownership_provider)
            self.addClass(rsignature)
            fn.return_type = rsignature
            fn.return_ownership = ownerships[ret_tv_info.ownership_var]
            for block in fn.body.blocks:
                for i in block.instructions:
                    if i.type is None:
                        continue
                    signature = Signatures.ClassInstantiationSignature()
                    signature.name = i.type.value
                    signature = Normalizer.normalizeClassOwnershipSignature(signature, 
                                                                            i.tv_info,
                                                                            ownership_dep_map,
                                                                            members,
                                                                            ownership_provider)
                    i.type_signature = signature
                    i.ownership = ownerships[i.tv_info.ownership_var]
                    self.addClass(signature)
                    if isinstance(i, IR.NamedFunctionCall):
                        if not i.ctor:
                            signature = Signatures.FunctionOwnershipSignature()
                            for arg in i.args:
                                arg_instr = fn.body.getInstruction(arg)
                                signature.args.append(arg_instr.tv_info)
                            signature.result = i.tv_info
                            signature.allocator = copy.deepcopy(fn.ownership_signature.allocator)
                            signature.name = i.name
                            signature = Normalizer.normalizeFunctionOwnershipSignature(signature, 
                                                                                       ownership_dep_map,
                                                                                       members,
                                                                                       ownership_provider, onlyBorrow=True)
                            i.name = signature
                            self.addFunction(signature)

    def processClass(self, signature):
        if signature not in self.classes:
            if signature.name == Util.getUnit():
                return
            #print("Processing class %s" % signature)
            clazz = self.program.classes[signature.name]
            clazz = copy.deepcopy(clazz)
            for borrow in signature.borrows:
                clazz.lifetimes.append("'l%s" % borrow.borrow_id.value)
            fields = []
            field_infos = {}
            for member in signature.members:
                if member.root == signature.root.group_var:
                    field_infos[member.kind.index] = member.info
            ownership_dep_map = MemberInfo.calculateOwnershipDepMap(signature.members)
            ownership_provider = Normalizer.OwnershipProvider()
            ownership_provider.borrow_list = signature.borrows
            allocator = copy.deepcopy(signature.allocator)
            for (index, f) in enumerate(clazz.fields):
                #print("process field", type(f.type.name))
                fsignature = Signatures.ClassInstantiationSignature()
                fsignature.name = f.type.name
                if index in field_infos:
                    info = field_infos[index]
                else:
                    info = allocator.nextTypeVariableInfo()
                fsignature = Normalizer.normalizeClassOwnershipSignature(fsignature, 
                                                                         info,
                                                                         ownership_dep_map,
                                                                         copy.deepcopy(signature.members),
                                                                         ownership_provider)
                f.type = fsignature
                if info.group_var in ownership_dep_map:
                    dep_ownership_vars = ownership_dep_map[info.group_var]
                    dep_lifetimes = []
                    for borrow in signature.borrows:
                        if borrow.ownership_var in dep_ownership_vars:
                            dep_lifetimes.append("'l%s" % borrow.borrow_id.value)
                    if len(dep_lifetimes) > 0:
                        f.dep_lifetimes = dep_lifetimes
                for borrow in signature.borrows:
                    if borrow.ownership_var == info.ownership_var:
                        f.lifetime = "'l%s" % borrow.borrow_id.value
                self.addClass(fsignature)
                fields.append(f)
            clazz.fields = fields
            self.classes[signature] = clazz

    def processQueue(self):
        while len(self.queue) > 0:
            first = self.queue.pop(0)
            if isinstance(first, Signatures.FunctionOwnershipSignature):
                self.processFunction(first)
            if isinstance(first, Signatures.ClassInstantiationSignature):
                self.processClass(first)

def monomorphize(program, profile_store):
    main_name = Util.QualifiedName("Main", "main")
    main_sig = Signatures.FunctionOwnershipSignature()
    main_sig.name = main_name
    main_sig.result = main_sig.allocator.nextTypeVariableInfo()
    monomorphizer = Monomorphizer(program, profile_store)
    monomorphizer.addFunction(main_sig)
    monomorphizer.processQueue()
    return (monomorphizer.classes, monomorphizer.functions)
