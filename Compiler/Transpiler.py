import Compiler.IR as IR
import Compiler.Util as Util
import Compiler.Typechecker as Typechecker
import Compiler.Ownership.Inference as Inference
import Compiler.Ownership.Signatures as Signatures
import Compiler.Ownership.Lifetime as Lifetime
import Compiler.Syntax as Syntax

def ii(id):
    id = "i_%s_%s" % (id.block, id.value)
    return id

def bi(id):
    id = "_block_%s" % id
    return id

def vi(v):
    if v.arg:
        return "arg_%s" % v.value
    else:
        return "tmp_%s" % v.value

class Transpiler(object):
    def __init__(self):
        self.output = None
        self.indentLevel = 0
        self.type_names = {}
        self.function_names = {}

    def initialize(self, classes, output):
        self.classes = classes
        self.output = open(output, "w")

    def print(self, m):
        self.output.write(m)

    def transpileType(self, type):
        if isinstance(type, Typechecker.NamedType):
            type = type.value
        if isinstance(type, Syntax.Type):
            type = type.name
        if isinstance(type, Signatures.ClassInstantiationSignature):
            type_name = type.name
            if str(type_name) == ".()":
                return "()"
            if str(type_name) == "Bool.Bool":
                return "bool"
            if type_name not in self.type_names:
                self.type_names[type_name] = []
            instances = self.type_names[type_name]
            for (index, i) in enumerate(instances):
                if i == type:
                    return "%s_%s_%s" % (type_name.moduleName, type_name.name, index)
            index = len(instances)
            instances.append(type)
            return "%s_%s_%s" % (type_name.moduleName, type_name.name, index)
        if str(type) == ".()":
            return "()"
        name = str(type)
        name = name.replace(".", "_")
        return name

    def transpileFnName(self, sig):
        if sig.name not in self.function_names:
            self.function_names[sig.name] = []
        instances = self.function_names[sig.name]
        for (index, i) in enumerate(instances):
            if i == sig:
                return "%s_%s_%s" % (sig.name.moduleName, sig.name.name, index)
        index = len(instances)
        instances.append(sig)
        return "%s_%s_%s" % (sig.name.moduleName, sig.name.name, index)
    
    def getIndent(self):
        indent = self.indentLevel * " "
        return indent
    
    def addInstr(self, i, value, partial=False):
        indent = self.indentLevel * " "
        ty = self.transpileType(i.type_signature)
        prefix = ""
        if isinstance(i.ownership, Inference.Borrow):
            ty = "&%s" % ty
            if isinstance(i, IR.ValueRef):
                prefix = "&"
        if partial:
            self.print("%slet %s : %s = %s%s\n" % (self.getIndent(), ii(i.id), ty, prefix, value))
        else:
            self.print("%slet %s : %s = %s%s;\n" % (self.getIndent(), ii(i.id), ty, prefix, value))

    def processBlock(self, fn, block_id):
        self.print("%slet %s = {\n" % (self.getIndent(), bi(block_id)));
        b = fn.body.getBlock(block_id)
        self.transpileBlock(fn, b)
        self.print("%s};\n" % self.getIndent());

    def transpileBlock(self, fn, block):
        self.indentLevel += 4
        for i in block.instructions:
            if isinstance(i, IR.NamedFunctionCall):
                if i.name.name == Util.getUnit():
                    self.addInstr(i, "()")
                else:
                    if i.ctor:
                        clazz = self.classes[i.type_signature]
                        call_args = []
                        for (index, arg) in enumerate(i.args):
                            field = clazz.fields[index]
                            call_args.append("%s: %s" % (field.name, ii(arg)))
                        call_args = ", ".join(call_args)
                        self.addInstr(i, "%s{%s}" % (self.transpileType(i.type_signature), call_args))
                    else:    
                        call_args = []
                        for arg in i.args:
                            call_args.append("%s" % ii(arg))
                        call_args = ", ".join(call_args)
                        self.addInstr(i, "%s(%s)" % (self.transpileFnName(i.name), call_args))
            elif isinstance(i, IR.Bind):
                self.print("%slet %s = %s;\n" % (self.getIndent(), vi(i.name), ii(i.rhs)))
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.BlockRef):
                self.processBlock(fn, i.value)
                self.addInstr(i, "%s" % (bi(i.value)))
            elif isinstance(i, IR.BoolLiteral):
                if i.value:
                    self.addInstr(i, "true")
                else:
                    self.addInstr(i, "false")
            elif isinstance(i, IR.ValueRef):
                v = vi(i.name)
                for field in i.fields:
                    v += ".%s" % field
                if i.clone:
                    v = v + ".clone()"
                self.addInstr(i, v)
            elif isinstance(i, IR.Nop):
                pass
            elif isinstance(i, IR.If):
                tb = fn.body.getBlock(i.true_branch)
                fb = fn.body.getBlock(i.false_branch)
                self.addInstr(i, "if %s {" % (ii(i.cond)), partial=True)
                self.transpileBlock(fn, tb)
                self.print("%s} else {\n" % self.getIndent())
                self.transpileBlock(fn, fb)
                self.print("%s};\n" % self.getIndent())
            else:
                Util.error("Transpiler not handling %s" % i)
        last = block.getLastReal()
        self.print("%s%s\n" % (self.getIndent(), ii(last.id)))
        self.indentLevel -= 4

    def transpileFn(self, sig, fn):
        fn_args = []
        lifetimes = []
        for arg in fn.args:
            ty = self.transpileType(arg.type)
            if len(arg.dep_lifetimes) > 0:
                lifetimes += arg.dep_lifetimes
                ty = "%s<%s>" % (ty, Lifetime.asList(arg.dep_lifetimes))
            if arg.lifetime:
                ty = "&%s %s" % (arg.lifetime, ty)
                lifetimes.append(arg.lifetime)
            fn_args.append("%s: %s" % (vi(arg.name), ty))
        fn_args = ", ".join(fn_args)
        fn_result = self.transpileType(fn.return_type)
        if len(fn.return_dep_lifetimes) > 0:
            lifetimes += fn.return_dep_lifetimes
            fn_result = "%s<%s>" % (fn_result, Lifetime.asList(fn.return_dep_lifetimes))
        if fn.return_lifetime:
            lifetimes.append(fn.return_lifetime)
            fn_result = "&%s %s" % (fn.return_lifetime, fn_result)
        if len(lifetimes) > 0:
            lifetimes = list(set(lifetimes))
            lifetime_deps = {}
            for lifetime in lifetimes:
                lifetime_deps[lifetime] = set()
            for (f, t) in fn.lifetime_dependencies:
                lifetime_deps[t].add(f)
            lifetime_decls = []
            for (l, deps) in lifetime_deps.items():
                decl = "%s: %s" % (l, Lifetime.asList(deps))
                lifetime_decls.append(decl)
            self.print("fn %s<%s>(%s) -> %s {\n" % (self.transpileFnName(sig), ", ".join(lifetime_decls), fn_args, fn_result))
        else:
            self.print("fn %s(%s) -> %s {\n" % (self.transpileFnName(sig), fn_args, fn_result))
        first_block = fn.body.getFirst()
        self.transpileBlock(fn, first_block)    
        self.print("}\n\n")

    def transpileClass(self, sig, c):
        if sig.name == Util.getBool():
            return
        self.print("#[derive(Clone)]\n")
        self.print("struct %s<%s> {\n" % (self.transpileType(sig), Lifetime.asList(c.lifetimes)))
        for field in c.fields:
            dep_lifetimes = ""
            if field.dep_lifetimes is not None:
                dep_lifetimes = "<%s>" % (Lifetime.asList(field.dep_lifetimes))
            if field.lifetime:
                self.print("    %s: &%s %s%s,\n" % (field.name, field.lifetime, self.transpileType(field.type), dep_lifetimes))
            else:
                self.print("    %s: %s%s,\n" % (field.name, self.transpileType(field.type), dep_lifetimes))
        self.print("}\n\n")

def transpile(classes, functions, output):
    transpiler = Transpiler()
    transpiler.initialize(classes, output)
    transpiler.print("#![allow(non_camel_case_types)]\n")
    transpiler.print("#![allow(unused_variables)]\n")
    transpiler.print("#![allow(dead_code)]\n")
    transpiler.print("#![allow(non_snake_case)]\n")
    transpiler.print("\n\n")
    for (sig, c) in classes.items():
        transpiler.transpileClass(sig, c)
    for (sig, f) in functions.items():
        transpiler.transpileFn(sig, f)
    transpiler.print("fn main() {\n")
    transpiler.print("    Main_main_0();\n")
    transpiler.print("}\n")
    transpiler.print("\n\n")
    transpiler.output.close()