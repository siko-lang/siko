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

unitType = NamedType()
unitType.value = Util.getUnit()
boolType = NamedType()
boolType.value = Util.getBool()

class TypeVar(object):
    def __init__(self):
        self.value = None
    
    def __str__(self):
        return "$tv.%s" % self.value

class Typechecker(object):
    def __init__(self):
        self.program = None
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
                elif isinstance(i, IR.Loop):
                    v = self.getNextVar()
                    self.types[i.var] = v
                v = self.getNextVar()
                self.types[i.id] = v
                #print("Initializing %s = %s" % (i.id, v))

    def unify(self, type1, type2):
        #print("Unifying %s/%s" % (type1, type2))
        type1 = self.substitution.apply(type1)
        type2 = self.substitution.apply(type2)
        #print("Unifying2 %s/%s" % (type1, type2))
        if isinstance(type1, TypeVar):
            self.substitution.add(type1, type2)
        elif isinstance(type2, TypeVar):
            self.substitution.add(type2, type1)
        elif isinstance(type1, NamedType) and isinstance(type2, NamedType):
            if type1.value != type2.value:
                print("Type mismatch named %s/%s" % (type(type1.value), type(type2.value)))
                Util.error("Type mismatch named %s/%s" % (type1, type2))
        else:
            Util.error("Type mismatch %s/%s" % (type1, type2))

    def check(self, fn):
        block = fn.body.getFirst()
        self.checkBlock(block, fn)
        returnType = NamedType()
        returnType.value = fn.return_type.name.name
        self.unify(self.types[block.getLast().id], returnType)

    def getFieldType(self, ty, field_name):
        if isinstance(ty, NamedType):
            clazz = self.program.classes[ty.value]
            found = False
            for field in clazz.fields:
                if field.name == field_name:
                    found = True
                    fieldType = NamedType()
                    fieldType.value = field.type.name.name
                    #print("field type %s [%s]" % (fieldType, i.name))
                    return fieldType
            if not found:
                Util.error("field %s not found on %s" % (field_name, ty.value))
        Util.error("field %s not found on %s" % (field_name, ty))

    def checkInstruction(self, block, fn, i):
        if isinstance(i, IR.BlockRef):
            block = fn.body.getBlock(i)
            self.checkBlock(block, fn)
            last = block.getLast()
            self.unify(self.types[last.id], self.types[i.id])
        elif isinstance(i, IR.Return):
            returnType = NamedType()
            returnType.value = fn.return_type.name.name
            self.unify(self.types[i.arg], returnType)
        elif isinstance(i, IR.DropVar):
            pass
        elif isinstance(i, IR.Break):
            pass # TODO
            #self.unify(self.types[i.arg], returnType)
        elif isinstance(i, IR.Continue):
            pass # TODO
            #self.unify(self.types[i.arg], returnType)
        elif isinstance(i, IR.Loop):
            body_block = fn.body.getBlock(i.body)
            self.checkBlock(body_block, fn)
            last = body_block.getLast()
            self.unify(self.types[i.init], self.types[i.var])
            self.unify(self.types[i.init], self.types[last.id])
        elif isinstance(i, IR.NamedFunctionCall):
            #print("Checking function call for %s" % i.name)
            #print("%s" % i.name.item.return_type.name)
            returnType = NamedType()
            if i.name in self.program.functions:
                item = self.program.functions[i.name]
                if len(item.args) != len(i.args):
                    Util.error("fn %s expected %d args found %d" % (i.name, len(item.args), len(i.args)))
                for (index, i_arg) in enumerate(i.args):
                    fn_arg = item.args[index]
                    arg_type = NamedType()
                    arg_type.value = fn_arg.type.name
                    self.unify(self.types[i_arg], arg_type)
                returnType.value = item.return_type.name.name
            elif i.name in self.program.classes:
                clazz = self.program.classes[i.name]
                if len(clazz.fields) != len(i.args):
                    Util.error("clazz ctor %s expected %d args found %d" % (i.name, len(clazz.fields), len(i.args)))
                for (index, i_arg) in enumerate(i.args):
                    fn_arg = clazz.fields[index]
                    arg_type = NamedType()
                    arg_type.value = fn_arg.type.name.name
                    self.unify(self.types[i_arg], arg_type)
                returnType.value = i.name
            else:
                (clazz, method_name) = i.name.name.split(".")
                clazz_name = Util.QualifiedName()
                clazz_name.module = i.name.module
                clazz_name.name = clazz
                clazz = self.program.classes[clazz_name]
                for method in clazz.methods:
                    if method.name == method_name:
                        if len(method.args) != len(i.args):
                            Util.error("fn %s expected %d args found %d" % (i.name, len(method.args), len(i.args)))
                        for (index, i_arg) in enumerate(i.args):
                            fn_arg = method.args[index]
                            arg_type = NamedType()
                            arg_type.value = fn_arg.type.name
                            self.unify(self.types[i_arg], arg_type)
                        returnType.value = method.return_type.name.name
            #print("return type %s [%s]" % (returnType, i.name))
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
            true_block = fn.body.getBlock(i.true_branch)
            self.checkBlock(true_block, fn)
            true_block_last = true_block.getLast()
            self.unify(self.types[true_block_last.id], self.types[i.id])
            false_block = fn.body.getBlock(i.false_branch)
            self.checkBlock(false_block, fn)
            false_block_last = false_block.getLast()
            self.unify(self.types[false_block_last.id], self.types[i.id])
        elif isinstance(i, IR.MethodCall):
            ty = self.types[i.receiver]
            ty = self.substitution.apply(ty)
            if isinstance(ty, NamedType):
                clazz = self.program.classes[ty.value]
                found = False
                total_args = [i.receiver] + i.args
                for method in clazz.methods:
                    if method.name == i.name:
                        if len(method.args) != len(total_args):
                            Util.error("method %s expected %d args found %d" % (method.name, len(method.args), len(total_args)))
                        for (index, i_arg) in enumerate(total_args):
                            method_arg = method.args[index]
                            arg_type = NamedType()
                            arg_type.value = method_arg.type.name
                            self.unify(self.types[i_arg], arg_type)
                        found = True
                        returnType = NamedType()
                        returnType.value = method.return_type.name.name
                        #print("method return type %s [%s]" % (returnType, i.name))
                        self.unify(self.types[i.id], returnType)
                if not found:
                    Util.error("method %s not found on %s" % (i.name, ty.value))
        elif isinstance(i, IR.MemberAccess):
            ty = self.types[i.receiver]
            ty = self.substitution.apply(ty)
            fieldType = self.getFieldType(ty, i.name)
            self.unify(self.types[i.id], fieldType)
        elif isinstance(i, IR.ValueRef):
            currentType = self.types[i.name]
            currentType = self.substitution.apply(currentType)
            for field in i.fields:
                currentType = self.getFieldType(currentType, field)
            self.unify(self.types[i.id], currentType)
        else:
            print("Typecheck not handled", type(i))

    def checkBlock(self, block, fn):
        for i in block.instructions:
            self.checkInstruction(block, fn, i)

    def finalize(self, fn):
        for block in fn.body.blocks:
            #print("#%d. block:" % block.id)
            for i in block.instructions:
                type = self.types[i.id]
                type = self.substitution.apply(type)
                #print("%5s %30s : %s" % (i.id, i, type))

def checkFunction(f, program):
    checker = Typechecker()
    checker.program = program
    #print("Type checking %s" % f.name)
    checker.initialize(f)
    checker.check(f)
    checker.finalize(f)

def checkProgram(program):
    for m in program.modules:
        for item in m.items:
            if isinstance(item, Syntax.Function):
                checkFunction(item, program)
            if isinstance(item, Syntax.Class):
                for m in item.methods:
                    checkFunction(m, program)