import Compiler.Syntax as Syntax
import Compiler.IR as IR
import Compiler.Util as Util

nextVar = 0

class Environment(object):
    def __init__(self):
        self.varList = []
        self.vars = {}
        self.parent = None

    def addVar(self, var, argIndex = None, bind_id = None):
        global nextVar
        tmpvar = IR.TempVar()
        if argIndex is not None:
            tmpvar.value = argIndex
            tmpvar.arg = True
        else:
            nextVar+=1
            tmpvar.value = nextVar
        self.vars[var] = (tmpvar, bind_id)
        self.varList.append((tmpvar, bind_id))
        return self.vars[var][0]

    def resolveVar(self, var):
        if var in self.vars:
            return self.vars[var]
        else:
            if self.parent:
                return self.parent.resolveVar(var)
            else:
                return None

class ResolvedItem(object):
    def __init__(self):
        self.name = None
        self.item = None

    def __str__(self):
        return str(self.name)

class ModuleResolver(object):
    def __init__(self):
        self.module = None
        self.localItems = {}
        self.importedItems = {}

    def addLocalItem(self, name, item):
        if name not in self.localItems:
            self.localItems[name] = []
        resolvedItem = ResolvedItem()
        qualifiedName = Util.QualifiedName(self.module, name)
        resolvedItem.name = qualifiedName
        resolvedItem.item = item
        self.localItems[name].append(resolvedItem)

    def addImportedItem(self, name, item):
        #print("addImportedItem ", name, type(name))
        if name not in self.importedItems:
            self.importedItems[name] = []
        self.importedItems[name].append(item)

    def resolveName(self, name):
        #print("resolveName ", name, type(name))
        if name in self.localItems:
            items = self.localItems[name]
            if len(items) > 1:
                Util.error("%s is ambiguous" % name)
            return items[0]
        else:
            if name in self.importedItems:
                items = self.importedItems[name]
                if len(items) > 1:
                    Util.error("%s is ambiguous" % name)
                return items[0]
            else:
                return None

class Resolver(object):
    def __init__(self):
        self.modules = []
        self.moduleResolvers = {}

    def resolveFunction(self, moduleName, fn):
        # print("Resolving fn %s" % fn.name)
        moduleResolver = self.moduleResolvers[moduleName]
        env = Environment()
        for (index, arg) in enumerate(fn.args):
            arg.name = env.addVar(arg.name, argIndex=index)
            arg_type = moduleResolver.resolveName(arg.type.name)
            if arg_type is None:
                Util.error("Failed to resolve type %s" % arg.type.name)
            else:
                arg.type = arg_type
        if fn.return_type.name == "()":
            fn.return_type = Util.getUnit()
        else:
            fn.return_type = moduleResolver.resolveName(fn.return_type.name)
        block = fn.body.getFirst()
        self.resolveBlock(env, moduleResolver, block, fn)
    
    def resolveBlock(self, penv, moduleResolver, block, fn):
        #print("Processing block ", block.id)
        env = Environment()
        env.parent = penv
        for instruction in block.instructions:
            if isinstance(instruction, IR.Bind):
                instruction.name = env.addVar(instruction.name, bind_id=instruction.id)
            elif isinstance(instruction, IR.BlockRef):
                b = fn.body.getBlock(instruction)
                self.resolveBlock(env, moduleResolver, b, fn)
            elif isinstance(instruction, IR.If):
                b = fn.body.getBlock(instruction.true_branch)
                self.resolveBlock(env, moduleResolver, b, fn)
                b = fn.body.getBlock(instruction.false_branch)
                self.resolveBlock(env, moduleResolver, b, fn)
            elif isinstance(instruction, IR.Loop):
                loop_env = Environment()
                loop_env.parent = env
                instruction.var = loop_env.addVar(instruction.var, bind_id=instruction.id)
                b = fn.body.getBlock(instruction.body)
                self.resolveBlock(env, moduleResolver, b, fn)
            elif isinstance(instruction, IR.ValueRef):
                var = env.resolveVar(instruction.name)
                if var:
                    instruction.name = var[0]
                    instruction.bind_id = var[1]
                else:
                    Util.error("Undefined var %s" % instruction.name)
            elif isinstance(instruction, IR.NamedFunctionCall):
                if instruction.name == Util.getUnit():
                    pass # TODO
                else:
                    var = env.resolveVar(instruction.name)
                    if var:
                        instruction.name = var[0]
                    else:
                        item = moduleResolver.resolveName(instruction.name)
                        if item:
                            instruction.name = item.name
                        else:
                            Util.error("Unknown fn %s %s" % (instruction.name, type(instruction.name)))
        vars = env.varList
        vars.reverse()
        for (var, bind_id) in vars:
            i = IR.DropVar()
            i.name = var
            i.bind_id = bind_id
            block.addInstruction(i)
        # fn.body.dump()

    def resolveClass(self, moduleName, clazz, ir_program):
        moduleResolver = self.moduleResolvers[moduleName]
        for f in clazz.fields:
            f.type.name = moduleResolver.resolveName(f.type.name)
        for m in clazz.methods:
            methodName = Util.QualifiedName(moduleName, m.name, clazz.name)
            ir_program.functions[methodName] = m
            self.resolveFunction(moduleName, m)

    def getModuleResolver(self, name):
        if name not in self.moduleResolvers:
            resolver = ModuleResolver()
            resolver.module = name
            self.moduleResolvers[name] = resolver
        return self.moduleResolvers[name]

    def localItems(self, module):
        resolver = self.getModuleResolver(module.name)
        for item in module.items:
            if isinstance(item, Syntax.Function):
                resolver.addLocalItem(item.name, item)
            if isinstance(item, Syntax.Class):
                for method in item.methods:
                    name = "%s.%s" % (item.name, method.name)
                    resolver.addLocalItem(name, method)    
                resolver.addLocalItem(item.name, item)

    def processImports(self, program):
        for m in program.modules:
            for importItem in m.items:
                if isinstance(importItem, Syntax.Import):
                    sourceResolver = self.getModuleResolver(importItem.module)
                    targetResolver = self.getModuleResolver(m.name)
                    if importItem.alias:
                        for (name, items) in sourceResolver.localItems.items():
                            for item in items:
                                name = importItem.alias + "." + name
                                targetResolver.addImportedItem(name, item)
                    else:
                        for (name, items) in sourceResolver.localItems.items():
                            for item in items:
                                targetResolver.addImportedItem(name, item)
                            for item in items:
                                fullName = "%s.%s" % (sourceResolver.module, name)
                                targetResolver.addImportedItem(fullName, item)
    
    def resolve(self, program):
        ir_program = IR.Program()
        for m in program.modules:
            self.localItems(m)
        self.processImports(program)
        for moduleName, resolver in self.moduleResolvers.items():
            for (name, items) in resolver.localItems.items():
                for item in items:
                    resolver.addImportedItem("%s.%s" % (moduleName, name), item)
        for m in program.modules:
            # print("Processing m", m.name, len(m.items))
            for item in m.items:
                if isinstance(item, Syntax.Function):
                    qualifiedName = Util.QualifiedName(m.name, item.name)
                    ir_program.functions[qualifiedName] = item
                    self.resolveFunction(m.name, item)
                if isinstance(item, Syntax.Class):
                    qualifiedName = Util.QualifiedName(m.name, item.name)
                    ir_program.classes[qualifiedName] = item
                    self.resolveClass(m.name, item, ir_program)
        return ir_program