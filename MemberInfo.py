
class MemberKind(object):
    def __init__(self):
        self.type = None
        self.index = 0

    
class MemberInfo(object):
    def __init__(self):
        self.root = None
        self.kind = None
        self.info = None

    def __str__(self):
        return "%s/%s.%s/%s" % (self.kind.type, self.root, self.kind.index, self.info)

    def __repr__(self) -> str:
        return self.__str__()
