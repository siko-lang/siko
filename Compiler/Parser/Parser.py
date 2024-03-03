import Compiler.Lexer as Lexer
import Compiler.Parser.Module as Module
import Compiler.Util as Util
import Compiler.Token as Token

class Parser(object):
    def __init__(self):
        self.modules = []
        self.tokens = []
        self.index = 0

    def step(self):
        self.index += 1

    def error(self, msg):
        print("Parsing ", self.filename)
        self.dump()
        Util.error(msg)

    def dump(self):
        for i in range(0, self.index):
            if self.tokens[i].value:
                print(self.tokens[i].value, end =" ")
            else:
                print(self.tokens[i].type, end =" ")
        print("")

    def expect(self, ty):
        if isinstance(ty, Token.Token):
            if self.tokens[self.index].type == ty.type:
                self.step()
                return
        if self.tokens[self.index].type == ty:
            self.step()
        else:
            if self.tokens[self.index].value is None:
                self.error("Expected %s found %s" % (ty, self.tokens[self.index].type))
            else:
                self.error("Expected %s found %s/%s" % (ty, self.tokens[self.index].type, self.tokens[self.index].value))

    def peek(self, ty):
        if isinstance(ty, Token.Token):
            return self.tokens[self.index].type == ty.type
        return self.tokens[self.index].type == ty

    def parseQualifiedName(self):
        if self.peek("typeid"):
            name = self.parseTypeName()
            while self.peek(Token.Dot()):
                self.expect(Token.Dot())
                name += "."
                if self.peek("varid"):
                    name += self.parseName()
                    break
        return name

    def parseModuleName(self):
        name = self.parseTypeName()
        while self.peek(Token.Dot()):
            self.expect(Token.Dot())
            n = self.parseTypeName()
            name += n
        return name

    def parseName(self):
        name = self.tokens[self.index].value
        self.expect("varid")
        return name

    def parseTypeName(self):
        name = self.tokens[self.index].value
        self.expect("typeid")
        return name

    def maybeParseSemicolon(self):
        if self.peek(Token.Semicolon()):
            self.expect(Token.Semicolon())
            return True
        else:
            return False

    def parse(self, program, filename):
        self.filename = filename
        f = open(filename)
        chars = []
        for line in f.readlines():
            chars = chars + [*line]
        lexer = Lexer.Lexer()
        self.tokens = lexer.lex(chars)
        #print("Tokens", self.tokens)
        while self.index < len(self.tokens):
            m = Module.parseModule(self)
            program.modules.append(m)
        #s = json.dumps(program, default=Syntax.default_serializer, indent=2)
        #print(s)
        f.close()