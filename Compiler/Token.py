class Token:
    def __init__(self, ty, value):
        self.type = ty
        self.value = value

    def __repr__(self):
        if self.value:
            return "%s:%s" % (self.type, self.value)
        else:
            return self.type

def VarIdentifier(s):
    return Token("varid", s)

def TypeIdentifier(s):
    return Token("typeid", s)

def String(s):
    return Token("string", s)

def Keyword(s):
    return Token(s, None)

def Number(i):
    return Token("number", i)

def Dot():
    return Token("dot", None)

def LeftParen():
    return Token("leftparen", None)

def RightParen():
    return Token("rightparen", None)

def LeftCurly():
    return Token("leftcurly", None)

def RightCurly():
    return Token("rightcurly", None)

def LeftBracket():
    return Token("leftbracket", None)

def RightBracket():
    return Token("rightbracket", None)

def Semicolon():
    return Token("semicolon", None)

def Colon():
    return Token("colon", None)

def At():
    return Token("@", None)

def Comma():
    return Token("comma", None)

def Equal():
    return Token("equal", None)

def NotEqual():
    return Token("notequal", None)

def Plus():
    return Token("plus", None)

def Minus():
    return Token("minus", None)

def RightArrow():
    return Token("rightarrow", None)

def RightDoubleArrow():
    return Token("rightdoublearrow", None)

def GreaterThan():
    return Token("greaterthan", None)

def LessThan():
    return Token("lessthan", None)

def Wildcard():
    return Token("wildcard", None)

def ExclamationMark():
    return Token("exclamationmark", None)
