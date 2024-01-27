import sys

class QualifiedName(object):
    def __init__(self):
        self.module = None
        self.name = None

    def __str__(self):
        return "%s.%s" % (self.module, self.name)

    def __eq__(self, other):
        """Overrides the default implementation"""
        if isinstance(other, QualifiedName):
            return self.name == other.name and self.module == other.module
        return False
    
    def __ne__(self, other):
        """Overrides the default implementation (unnecessary in Python 3)"""
        return not self.__eq__(other)

    def __hash__(self):
        return hash(self.__str__())

def error(msg):
    print(msg)
    sys.exit(1)

def getBool():
    name = QualifiedName()
    name.module = "Bool"
    name.name = "Bool"
    return name

def getUnit():
    name = QualifiedName()
    name.module = "Unit"
    name.name = "Unit"
    return name
