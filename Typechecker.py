import Syntax
import IR
import Util

class Substitution(object):
    def __init__(self):
        self.substitutions = {}

    def add(self, var, type):
        self.substitutions[var] = type

    def apply(self, ty):
        res = ty
        while True:
            if isinstance(res, TypeVar):
                if res in self.substitutions:
                    res = self.substitutions[res]
                else:
                    return res
            else:
                return res

class NamedType(object):
    def __init__(self):
        self.value = None

    def __str__(self):
        return "Named:%s" % self.value

class TypeVar(object):
    def __init__(self):
        self.value = None
    
    def __str__(self):
        return "$tv.%s" % self.value

class Typechecker(object):
    def __init__(self):
        self.substitution = Substitution()
        self.nextVar = 0
        self.types = {}

    def getNextVar(self):
        v = TypeVar()
        v.value = self.nextVar
        self.nextVar += 1
        return v

    def initialize(self, fn):
        for arg in fn.args:
            namedType = NamedType()
            namedType.value = arg.type.name
            self.types[arg.name] = namedType
        for block in fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.Bind):
                    v = self.getNextVar()
                    self.types[i.name] = v
                v = self.getNextVar()
                self.types[i.id] = v
                #print("Initializing %s = %s" % (i.id, v))

    def unify(self, type1, type2):
        #print("Unifying %s/%s" % (type1, type2))
        type1 = self.substitution.apply(type1)
        type2 = self.substitution.apply(type2)
        print("Unifying2 %s/%s" % (type1, type2))
        if isinstance(type1, TypeVar):
            self.substitution.add(type1, type2)
        elif isinstance(type2, TypeVar):
            self.substitution.add(type2, type1)
        elif isinstance(type1, NamedType) and isinstance(type2, NamedType):
            if type1.value != type2.value:
                Util.error("Type mismatch %s/%s" % (type1, type2))
        else:
            Util.error("Type mismatch %s/%s" % (type1, type2))

    def check(self, fn):
        block = fn.body.getFirst()
        self.checkBlock(block, fn)

    def checkBlock(self, block, fn):
        unitType = NamedType()
        unitType.value = "Main.Unit"
        boolType = NamedType()
        boolType.value = "Main.Bool"
        for i in block.instructions:
            if isinstance(i, IR.BlockRef):
                block = fn.body.getBlock(i)
                self.checkBlock(block, fn)
                if len(block.instructions) > 0:
                    last = block.instructions[-1]
                self.unify(self.types[last.id], self.types[i.id])
            elif isinstance(i, IR.NamedFunctionCall):
                #print("Checking function call for %s" % i.name)
                #print("%s" % i.name.item.return_type.name)
                returnType = NamedType()
                if isinstance(i.name.item, Syntax.Function):
                    returnType.value = i.name.item.return_type.name
                if isinstance(i.name.item, Syntax.Class):
                    returnType.value = i.name
                self.unify(self.types[i.id], returnType)
            elif isinstance(i, IR.Bind):
                self.unify(self.types[i.name], self.types[i.rhs])
                self.unify(self.types[i.id], unitType)
            elif isinstance(i, IR.VarRef):
                self.unify(self.types[i.id], self.types[i.name])
            elif isinstance(i, IR.BoolLiteral):
                self.unify(self.types[i.id], boolType)
            elif isinstance(i, IR.If):
                self.unify(self.types[i.cond], boolType)
                self.unify(self.types[i.id], self.types[i.true_branch])
                self.unify(self.types[i.id], self.types[i.false_branch])
            else:
                print("Not handled", type(i))

    def finalize(self, fn):
        for block in fn.body.blocks:
            print("#%d. block:" % block.id)
            for i in block.instructions:
                type = self.types[i.id]
                type = self.substitution.apply(type)
                print("%5s %30s : %s" % (i.id, i, type))

def checkFunction(f):
    checker = Typechecker()
    print("Type checking %s" % f.name)
    checker.initialize(f)
    checker.check(f)
    checker.finalize(f)

def checkProgram(program):
    for m in program.modules:
        for item in m.items:
            if isinstance(item, Syntax.Function):
                checkFunction(item)
            if isinstance(item, Syntax.Class):
                for m in item.methods:
                    checkFunction(m)