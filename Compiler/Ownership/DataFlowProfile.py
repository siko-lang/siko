class DataFlowProfile(object):
    def __init__(self) -> None:
        self.paths = None
        self.signature = None

    def __str__(self):
        return "(%s/%s)" % (self.signature, self.paths)
    
    def __repr__(self) -> str:
        return self.__str__()
    