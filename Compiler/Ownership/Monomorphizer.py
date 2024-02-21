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
    def __init__(self) -> None:
        self.program = None
        self.functions = {}
        self.classes = {}
        self.queue = []

    def addFunction(self, signature):
        self.queue.append(signature)

    def processFunction(self, signature):
        if signature not in self.functions:
            if signature.name == Util.getUnit():
                return
            print("Processing fn %s" % signature)
            fn = self.program.functions[signature.name]
            fn = copy.deepcopy(fn)
            self.functions[signature] = fn
            fn.ownership_signature = copy.deepcopy(signature)
            equality = Equality.EqualityEngine()
            equality.process(fn)
            members = fn.getAllMembers()
            #fn.body.dump()
            #print("members", members)
            ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
            forbidden_borrows = ForbiddenBorrows.ForbiddenBorrowsEngine()
            forbidden_borrows.process(fn, ownership_dep_map)
            inference = Inference.InferenceEngine()
            ownerships = inference.infer(fn, self.program.classes)
            for block in fn.body.blocks:
                for i in block.instructions:
                    print("i", i)
                    print("type!!", i.type)
                    print("type!!", type(i.type))
                    print("info %s", i.tv_info)
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
                                                                                       ownerships)
                            self.addFunction(signature)

    def processClass(self, signature):
        print("Processing class %s" % signature)

    def processQueue(self):
        while len(self.queue) > 0:
            first = self.queue.pop(0)
            if isinstance(first, Signatures.FunctionOwnershipSignature):
                self.processFunction(first)
            if isinstance(first, Signatures.ClassInstantiationSignature):
                self.processClass(first)

def monomorphize(program):
    main_name = Util.QualifiedName("Main", "main")
    main_sig = Signatures.FunctionOwnershipSignature()
    main_sig.name = main_name
    main_sig.result = main_sig.allocator.nextTypeVariableInfo()
    monomorphizer = Monomorphizer()
    monomorphizer.program = program
    monomorphizer.addFunction(main_sig)
    monomorphizer.processQueue()
