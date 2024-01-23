import Syntax

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

class Resolver(object):
    def __init__(self):
        self.modules = []
        self.moduleResolvers = {}

    def resolveFunction(self, fn):
        for arg in fn.args:
            pass

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
                    self.resolveFunction(item)