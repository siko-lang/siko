class NamedFunctionCall(object):
    def __init__(self):
        self.name = None
        self.args = []

class ValueCall(object):
    def __init__(self):
        self.id = None
        self.args = []

class MethodCall(object):
    def __init__(self):
        self.receiver = None
        self.name = None
        self.args = []

class Bind(object):
    def __init__(self):
        self.name = None
        self.rhs = None

