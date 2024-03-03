import Compiler.Syntax.Data as Data
import Compiler.Parser.Type as Type
import Compiler.Parser.Function as Function
import Compiler.Token as Token

def parseEnumVariant(parser):
    variant = Data.Variant()
    variant.name = parser.parseTypeName()
    if parser.peek(Token.LeftParen()):
        parser.expect(Token.LeftParen())
        while True:
            item = Type.parseType(parser)
            variant.items.append(item)
            if parser.peek(Token.Comma()):
                parser.expect(Token.Comma())
            elif parser.peek(Token.RightParen()):
                break
        parser.expect(Token.RightParen())
    if parser.peek(Token.Comma()):
        parser.expect(Token.Comma())
    return variant

def parseEnum(parser):
    parser.expect("enum")
    enum = Data.Enum()
    enum.name = parser.parseTypeName()
    if parser.peek(Token.LeftBracket()):
        enum.generics = Type.parseGenericDeclaration(parser)
    parser.expect(Token.LeftCurly())
    while not parser.peek(Token.RightCurly()):
        variant = parseEnumVariant(parser)
        enum.variants.append(variant)
    parser.expect(Token.RightCurly())
    return enum

def parseClassField(parser):
    field = Data.Field()
    field.name = parser.parseName()
    parser.expect(Token.Colon())
    field.type = Type.parseType(parser)
    return field

def parseClass(parser, module_name, derives):
    c = Data.Class()
    c.module_name = module_name
    c.derives = derives
    parser.expect("class")
    c.name = parser.parseTypeName()
    if parser.peek(Token.LeftBracket()):
        c.generics = Type.parseGenericDeclaration(parser)
    parser.expect(Token.LeftCurly())
    while not parser.peek(Token.RightCurly()):
        if parser.peek("varid"):
            field = parseClassField(parser)
            c.fields.append(field)
            if parser.peek(Token.Comma()):
                parser.expect(Token.Comma())
        elif parser.peek("fn"):
            fn = parseClassMemberFunction(parser, module_name)
            c.methods.append(fn)
        elif parser.peek("implicit"):
            c.implicit_member = True
            parser.step()
        else:
            parser.error("expected class item found %s" % parser.tokens[parser.index].type)
    parser.expect(Token.RightCurly())
    return c

def parseClassMemberFunction(parser, module_name):
    return Function.parseFunction(parser, module_name)

def parseExternClass(parser, module_name, derives):
    parser.expect("extern")
    c = parseClass(parser, module_name, derives)
    c.extern = True
    return c
