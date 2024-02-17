import IR
import Util

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

    def initialize(self, program):
        self.program = program
        self.output = open("main.rs", "w")

    def print(self, m):
        self.output.write(m)

    def transpileType(self, type):
        name = str(type.name)
        name = name.replace(".", "_")
        return name

    def transpileFnName(self, n):
        name = str(n)
        name = name.replace(".", "_")
        return name
    
    def getIndent(self):
        indent = self.indentLevel * " "
        return indent
    
    def addInstr(self, i, value):
        indent = self.indentLevel * " "
        self.print("%slet %s = %s;\n" % (self.getIndent(), ii(i.id), value))

    def processBlock(self, fn, block_id):
        self.print("%slet %s = {\n" % (self.getIndent(), bi(block_id)));
        b = fn.body.getBlock(block_id)
        self.transpileBlock(fn, b)
        self.print("%s};\n" % self.getIndent());

    def transpileBlock(self, fn, block):
        self.indentLevel += 4
        for i in block.instructions:
            if isinstance(i, IR.VarRef):
                self.addInstr(i, "%s" % vi(i.name))
            elif isinstance(i, IR.NamedFunctionCall):
                if i.ctor:
                    clazz = self.program.classes[i.name]
                    call_args = []
                    for (index, arg) in enumerate(i.args):
                        field = clazz.fields[index]
                        call_args.append("%s: %s" % (field.name, ii(arg)))
                    call_args = ", ".join(call_args)
                    self.addInstr(i, "%s{%s}" % (self.transpileFnName(i.name), call_args))
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
            elif isinstance(i, IR.Converter):
                self.addInstr(i, "/*convert*/%s" % (ii(i.arg)))
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
                self.addInstr(i, v)
            elif isinstance(i, IR.Nop):
                pass
            elif isinstance(i, IR.If):
                self.processBlock(fn, i.true_branch)
                self.processBlock(fn, i.false_branch)
                self.addInstr(i, "if %s { %s } else { %s }" % (ii(i.cond), bi(i.true_branch), bi(i.false_branch)))
            else:
                Util.error("Transpiler not handling %s" % i)
        last = block.getLastReal()
        self.print("%s%s\n" % (self.getIndent(), ii(last.id)))
        self.indentLevel -= 4

    def transpileFn(self, fn):
        fn_args = []
        for arg in fn.args:
            fn_args.append("%s: %s" % (vi(arg.name), self.transpileType(arg.type)))
        fn_args = ", ".join(fn_args)
        fn_result = self.transpileType(fn.return_type)
        self.print("fn %s_%s(%s) -> %s {\n" % (fn.module_name, fn.name, fn_args, fn_result))
        first_block = fn.body.getFirst()
        self.transpileBlock(fn, first_block)    
        self.print("}\n\n")

    def transpileClass(self, c):
        self.print("struct %s_%s {\n" % (c.module_name, c.name))
        for field in c.fields:
            self.print("    %s: %s,\n" % (field.name, self.transpileType(field.type)))
        self.print("}\n\n")

def transpile(program):
    transpiler = Transpiler()
    transpiler.initialize(program)
    transpiler.print("#![allow(non_camel_case_types)]\n")
    transpiler.print("#![allow(unused_variables)]\n")
    transpiler.print("\n\n")
    for c in program.classes.values():
        transpiler.transpileClass(c)
    for f in program.functions.values():
        transpiler.transpileFn(f)
    transpiler.print("fn main() {\n")
    transpiler.print("    Main_main();\n")
    transpiler.print("}\n")
    transpiler.print("\n\n")
    transpiler.output.close()