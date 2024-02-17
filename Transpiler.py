class Transpiler(object):
    def __init__(self):
        self.output = None

    def initialize(self):
        self.output = open("main.rs", "w")

    def print(self, m):
        self.output.write(m)

    def transpileType(self, type):
        name = str(type.name)
        name = name.replace(".", "_")
        return name

    def transpileFn(self, fn):
        fn_args = ""
        fn_result = self.transpileType(fn.return_type)
        self.print("fn %s(%s) -> %s {\n" % (fn.name, fn_args, fn_result))
        self.print("}\n\n")

    def transpileClass(self, c):
        self.print("struct %s {\n" % c.name)
        for field in c.fields:
            self.print("    %s: %s,\n" % (field.name, self.transpileType(field.type)))
        self.print("}\n\n")

def transpile(program):
    transpiler = Transpiler()
    transpiler.initialize()
    for c in program.classes.values():
        transpiler.transpileClass(c)
    for f in program.functions.values():
        transpiler.transpileFn(f)
    transpiler.output.close()