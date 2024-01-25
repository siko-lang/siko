import Syntax
import IR

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

class QualifiedName(object):
    def __init__(self):
        self.module = None
        self.name = None

    def __str__(self):
        return "%s.%s" % (self.module, self.name)

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
        qualifiedName = QualifiedName()
        qualifiedName.module = self.module
        qualifiedName.name = name
        resolvedItem.name = qualifiedName
        resolvedItem.item = item
        self.localItems[name].append(resolvedItem)

    def addImportedItem(self, name, item):
        if name not in self.importedItems:
            self.importedItems[name] = []
        self.importedItems[name].append(item)

    def resolveName(self, name):
        if name in self.localItems:
            items = self.localItems[name]
            if len(items) > 1:
                print("%s is ambiguous" % name)
            return items[0]
        else:
            if name in self.importedItems:
                items = self.importedItems[name]
                if len(items) > 1:
                    print("%s is ambiguous" % name)
                return items[0]
            else:
                return None


class Resolver(object):
    def __init__(self):
        self.modules = []
        self.moduleResolvers = {}

    def resolveFunction(self, moduleName, fn):
        moduleResolver = self.moduleResolvers[moduleName]
        envs = []
        envs.append(Environment())
        for arg in fn.args:
            arg.name = envs[-1].addVar(arg.name)
        for instruction in fn.body.instructions:
            if isinstance(instruction, IR.BlockBegin):
                env = Environment()
                env.parent = envs[-1]
                envs.append(env)
            if isinstance(instruction, IR.BlockEnd):
                envs.pop()
            if isinstance(instruction, IR.Bind):
                instruction.name = envs[-1].addVar(instruction.name)
            elif isinstance(instruction, IR.VarRef):
                instruction.name = envs[-1].resolveVar(instruction.name)
            elif isinstance(instruction, IR.NamedFunctionCall):
                var = envs[-1].resolveVar(instruction.name)
                if var:
                    instruction.name = var
                else:
                    item = moduleResolver.resolveName(instruction.name)
                    if item:
                        instruction.name = item
                    else:
                        print("Unknown fn %s" % instruction.name)
        fn.body.dump()

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
                                name = item.name.module + "." + name
                                targetResolver.addImportedItem(name, item)
    
    def resolve(self, program):
        for m in program.modules:
            self.localItems(m)
        self.processImports(program)
        for m in program.modules:
            for item in m.items:
                if isinstance(item, Syntax.Function):
                    self.resolveFunction(m.name, item)