class DataFlowProfileStore(object):
    def __init__(self):
        self.profiles = {}

    def addProfile(self, name, profile):
        self.profiles[name] = profile

    def getProfile(self, name):
        return self.profiles[name]
    