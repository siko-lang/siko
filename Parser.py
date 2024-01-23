import Lexer
import Token
import Syntax
import json

import sys

class Parser(object):
    def __init__(self):
        self.modules = []
        self.tokens = []
        self.index = 0

    def step(self):
        self.index += 1

    def error(self, msg):
        print(msg)
        assert False
        sys.exit(1)

    def expect(self, ty):
        if self.tokens[self.index].type == ty:
            self.step()
        else:
            if self.tokens[self.index].value is None:
                self.error("Expected %s found %s" % (ty, self.tokens[self.index].type))
            else:
                self.error("Expected %s found %s/%s" % (ty, self.tokens[self.index].type, self.tokens[self.index].value))

    def peek(self, ty):
        return self.tokens[self.index].type == ty

    def parseQualifiedName(self):
        if self.peek("typeid"):
            name = self.parseTypeName()
            while self.peek("dot"):
                self.expect("dot")
                name += "."
                if self.peek("varid"):
                    name += self.parseName()
                    break
        return name

    def parseModuleName(self):
        name = self.parseTypeName()
        while self.peek("dot"):
            self.expect("dot")
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

    def parseEnumVariant(self):
        name = self.parseName()
        self.expect("leftparen")
        arg = self.parseType()
        self.expect("rightparen")

    def parseEnum(self):
        self.expect("enum")
        name = self.parseName()
        self.expect("leftcurly")
        while not self.peek("rightcurly"):
            self.parseEnumVariant()
        self.expect("rightcurly")

    def parseImport(self):
        i = Syntax.Import()
        self.expect("import")
        i.module = self.parseModuleName()
        if self.peek("as"):
            self.expect("as")
            i.alias = self.parseTypeName()
        return i

    def parseItem(self):
        if self.peek("extern"):
            self.parseExternClass()
        elif self.peek("enum"):
            self.parseEnum()
        elif self.peek("class"):
            return self.parseClass()
        elif self.peek("import"):
            return self.parseImport()
        elif self.peek("fn"):
            return self.parseFunction()
        else:
            self.error("Expected module item, found %s" % self.tokens[self.index].type)

    def parseArgDef(self):
        name = self.parseName()
        self.expect("colon")
        ty = self.parseType()
        arg = Syntax.Arg()
        arg.name = name
        arg.type = ty
        return arg

    def parseFunctionArgs(self):
        self.expect("leftparen")
        args = []
        while not self.peek("rightparen"):
            args.append(self.parseExpr())
            if self.peek("rightparen"):
                break
            else:
                self.expect("comma")
        self.expect("rightparen")
        return args

    def parseFunctionCall(self):
        receiver = self.parsePrimary()
        while True:
            if self.peek("leftparen"):
                call = Syntax.FunctionCall()
                call.id = receiver
                call.args = self.parseFunctionArgs()
                receiver = call
            elif self.peek("dot"):
                self.expect("dot")
                name = self.parseName()
                if self.peek("leftparen"):
                    args = self.parseFunctionArgs()
                    m = Syntax.MemberCall()
                    m.receiver = receiver
                    m.name = name
                    m.args = args
                    receiver = m
                else:
                    m = Syntax.MemberAccess()
                    m.receiver = receiver
                    m.name = name
                    receiver = m
            else:
                break
        return receiver

    def parsePrimary(self):
        if self.peek("typeid"):
            name = self.parseQualifiedName()
            r = Syntax.TypeRef()
            r.name = name
            return r
        elif self.peek("varid"):
            name = self.parseName()
            r = Syntax.VarRef()
            r.name = name
            return r
        elif self.peek("leftcurly"):
            e = self.parseBlock()
            return e
        elif self.peek("break"):
            self.expect("break")
            e = Syntax.Break()
            e.arg = self.parseExpr()
            return e
        elif self.peek("continue"):
            self.expect("continue")
            e = Syntax.Continue()
            e.arg = self.parseExpr()
            return e
        elif self.peek("return"):
            self.expect("return")
            e = Syntax.Return()
            e.arg = self.parseExpr()
            return e
        elif self.peek("loop"):
            self.expect("loop")
            var = self.parseName()
            self.expect("equal")
            init = self.parseExpr()
            body = self.parseBlock()
            loop_expr = Syntax.Loop()
            loop_expr.var = var
            loop_expr.init = init
            loop_expr.body = body
            return loop_expr
        elif self.peek("if"):
            self.expect("if")
            if_expr = Syntax.If()
            if_expr.cond = self.parseExpr()
            if_expr.true_branch = self.parseBlock()
            if self.peek("else"):
                self.expect("else")
                if_expr.false_branch = self.parseBlock()
            return if_expr
        else:
            self.error("expected expr, found %s" % self.tokens[self.index].type)

    def parseExpr(self):
        return self.parseFunctionCall()

    def parseStatement(self):
        if self.peek("let"):
            self.expect("let")
            let_s = Syntax.LetStatement()
            let_s.var_name = self.parseName()
            self.expect("equal")
            let_s.rhs = self.parseExpr()
            return let_s
        else:
            expr = self.parseExpr()
            s = Syntax.ExprStatement()
            s.expr = expr
            return s

    def parseBlock(self):
        self.expect("leftcurly")
        block = Syntax.Block()
        while not self.peek("rightcurly"):
            s = self.parseStatement()
            block.statements.append(s)
            if self.peek("rightcurly"):
                break
            self.expect("semicolon")
        self.expect("rightcurly")
        return block

    def parseClassMemberFunction(self):
        self.parseFunction()

    def parseFunction(self):
        fn = Syntax.Function()
        self.expect("fn")
        name = self.parseName()
        fn.name = name
        self.expect("leftparen")
        while not self.peek("rightparen"):
            arg = self.parseArgDef()    
            fn.args.append(arg)    
            if not self.peek("rightparen"):
                self.expect("comma")
        self.expect("rightparen")
        self.expect("rightarrow")
        fn.return_type = self.parseType()
        if self.peek("equal"):
            self.expect("equal")
            self.expect("extern")
        else:
            fn.body = self.parseBlock()
        return fn

    def parseConstraints(self):
        self.expect("leftbracket")
        type_arg = self.parseName()
        self.expect("rightbracket")

    def parseType(self):
        name = self.parseQualifiedName()
        if self.peek("leftbracket"):
            self.expect("leftbracket")
            self.parseType()
            self.expect("rightbracket")
        ty = Syntax.Type()
        ty.name = name
        return ty

    def parseClassField(self):
        field = Syntax.Field()
        field.name = self.parseName()
        self.expect("colon")
        field.ty = self.parseType()
        return field

    def parseClass(self):
        c = Syntax.Class()
        self.expect("class")
        c.name = self.parseTypeName()
        if self.peek("leftbracket"):
            self.parseConstraints()
        self.expect("leftcurly")
        while not self.peek("rightcurly"):
            if self.peek("varid"):
                field = self.parseClassField()
                c.fields.append(field)
            elif self.peek("fn"):
                fn = self.parseClassMemberFunction()
                c.methods.append(fn)
            else:
                self.error("expected class item found %s" % self.tokens[self.index].type)
        self.expect("rightcurly")
        return c

    def parseExternClass(self):
        self.expect("extern")
        return self.parseClass()

    def parseModule(self):
        self.expect("module")
        name = self.parseModuleName()
        m = Syntax.Module()
        m.name = name
        self.expect("leftcurly")
        while not self.peek("rightcurly"):
            item = self.parseItem()
            if item:
                m.items.append(item)
        self.expect("rightcurly")
        return m

    def parse(self, program, filename):
        f = open(filename)
        chars = []
        for line in f.readlines():
            chars = chars + [*line]
        lexer = Lexer.Lexer()
        self.tokens = lexer.lex(chars)
        #print("Tokens", self.tokens)
        while self.index < len(self.tokens):
            m = self.parseModule()
            program.modules.append(m)
        #s = json.dumps(program, default=Syntax.default_serializer, indent=2)
        #print(s)
        f.close()