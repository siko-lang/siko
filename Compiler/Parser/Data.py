import Compiler.Syntax.Data as Data
import Compiler.Parser.Type as Type

def parseEnumVariant(parser):
    variant = Data.Variant()
    variant.name = parser.parseTypeName()
    if parser.peek("leftparen"):
        parser.expect("leftparen")
        while True:
            item = Type.parseType(parser)
            variant.items.append(item)
            if parser.peek("comma"):
                parser.expect("comma")
            elif parser.peek("rightparen"):
                break
        parser.expect("rightparen")
    if parser.peek("comma"):
        parser.expect("comma")
    return variant

def parseEnum(parser):
    parser.expect("enum")
    enum = Data.Enum()
    enum.name = parser.parseTypeName()
    if parser.peek("leftbracket"):
        enum.generics = Type.parseGenericDeclaration(parser)
    parser.expect("leftcurly")
    while not parser.peek("rightcurly"):
        variant = parseEnumVariant(parser)
        enum.variants.append(variant)
    parser.expect("rightcurly")
    return enum

def parseClassField(parser):
    field = Data.Field()
    field.name = parser.parseName()
    parser.expect("colon")
    field.type = Type.parseType(parser)
    return field

def parseClass(parser, module_name, derives):
    c = Data.Class()
    c.module_name = module_name
    c.derives = derives
    parser.expect("class")
    c.name = parser.parseTypeName()
    if parser.peek("leftbracket"):
        c.generics = Type.parseGenericDeclaration(parser)
    parser.expect("leftcurly")
    while not parser.peek("rightcurly"):
        if parser.peek("varid"):
            field = parseClassField(parser)
            c.fields.append(field)
            if parser.peek("comma"):
                parser.expect("comma")
        elif parser.peek("fn"):
            fn = parseClassMemberFunction(parser, module_name)
            c.methods.append(fn)
        else:
            parser.error("expected class item found %s" % parser.tokens[parser.index].type)
    parser.expect("rightcurly")
    return c

def parseClassMemberFunction(parser, module_name):
    return parser.parseFunction(module_name)

def parseExternClass(parser, module_name, derives):
    parser.expect("extern")
    return parseClass(parser, module_name, derives)
