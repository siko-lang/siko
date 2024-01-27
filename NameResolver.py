import Syntax
import IR
import Util

nextVar = 0

class Environment(object):
    def __init__(self):
        self.vars = {}
        self.parent = None

    def addVar(self, var):
        global nextVar
        tmpvar = IR.TempVar()
        tmpvar.value = nextVar
        self.vars[var] = tmpvar
        nextVar+=1
        return self.vars[var]

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
        qualifiedName = Util.QualifiedName()
        qualifiedName.module = self.module
        qualifiedName.name = name
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
        #print("Resolving fn %s" % fn.name)
        moduleResolver = self.moduleResolvers[moduleName]
        env = Environment()
        for arg in fn.args:
            arg.name = env.addVar(arg.name)
        fn.return_type.name = moduleResolver.resolveName(fn.return_type.name)
        block = fn.body.getFirst()
        self.resolveBlock(env, moduleResolver, block, fn)
    
    def resolveBlock(self, penv, moduleResolver, block, fn):
        #print("Processing block ", block.id)
        env = Environment()
        env.parent = penv
        for instruction in block.instructions:
            if isinstance(instruction, IR.Bind):
                instruction.name = env.addVar(instruction.name)
            if isinstance(instruction, IR.BlockRef):
                b = fn.body.getBlock(instruction)
                self.resolveBlock(env, moduleResolver, b, fn)
            elif isinstance(instruction, IR.VarRef):
                var = env.resolveVar(instruction.name)
                if var:
                    instruction.name = var
                else:
                    Util.error("Undefined var %s" % instruction.name)
            elif isinstance(instruction, IR.NamedFunctionCall):
                var = env.resolveVar(instruction.name)
                if var:
                    instruction.name = var
                else:
                    item = moduleResolver.resolveName(instruction.name)
                    if item:
                        instruction.name = item.name
                    else:
                        Util.error("Unknown fn %s %s" % (instruction.name, type(instruction.name)))
        #fn.body.dump()

    def resolveClass(self, moduleName, clazz):
        moduleResolver = self.moduleResolvers[moduleName]
        for f in clazz.fields:
            f.type.name = moduleResolver.resolveName(f.type.name)
        for m in clazz.methods:
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
        for m in program.modules:
            self.localItems(m)
        self.processImports(program)
        for moduleName, resolver in self.moduleResolvers.items():
            for (name, items) in resolver.localItems.items():
                for item in items:
                    resolver.addImportedItem("%s.%s" % (moduleName, name), item)
        for m in program.modules:
            for item in m.items:
                if isinstance(item, Syntax.Function):
                    qualifiedName = Util.QualifiedName()
                    qualifiedName.module = m.name
                    qualifiedName.name = item.name
                    program.functions[qualifiedName] = item
                    self.resolveFunction(m.name, item)
                if isinstance(item, Syntax.Class):
                    qualifiedName = Util.QualifiedName()
                    qualifiedName.module = m.name
                    qualifiedName.name = item.name
                    program.classes[qualifiedName] = item
                    self.resolveClass(m.name, item)