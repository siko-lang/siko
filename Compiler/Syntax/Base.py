class SerializableMixin:
    def to_dict(self):
        return {key: value for key, value in self.__dict__.items()}

def default_serializer(obj):
    if isinstance(obj, SerializableMixin):
        return obj.to_dict()
    raise TypeError(f"Object of type {obj.__class__.__name__} is not JSON serializable")

class SyntaxBase(SerializableMixin):
    def __init__(self):
        pass
