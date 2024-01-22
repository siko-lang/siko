import Syntax

class Resolver(object):
    def __init__(self):
        self.modules = []

    def resolveFunction(self, fn):
        for arg in fn.args:
            pass
        
    def resolve(self, program):
        for m in program.modules:
            for item in m.items:
                if isinstance(item, Syntax.Function):
                    self.resolveFunction(item)