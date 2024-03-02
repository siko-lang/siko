import Compiler.Syntax.Module as Module
import Compiler.Parser.Data as Data
import Compiler.Parser.Function as Function
import Compiler.Parser.Trait as Trait

def parseItem(parser, module_name):
    derives = []
    if parser.peek("@"):
        parser.expect("@")
        parser.expect("derive")
        parser.expect("leftparen")
        while True:
            d = parser.parseTypeName()
            derives.append(d)
            if parser.peek("rightparen"):
                break
            parser.expect("comma")
        parser.expect("rightparen")
    if parser.peek("extern"):
        Data.parseExternClass(parser, module_name, derives)
    elif parser.peek("enum"):
        return Data.parseEnum(parser)
    elif parser.peek("class"):
        return Data.parseClass(parser, module_name, derives)
    elif parser.peek("trait"):
        return Trait.parseTrait(parser, module_name)
    elif parser.peek("instance"):
        return Trait.parseInstance(parser, module_name)
    elif parser.peek("import"):
        return parseImport(parser)
    elif parser.peek("fn"):
        return Function.parseFunction(parser, module_name)
    else:
        parser.error("Expected module item, found %s/%s" % (parser.tokens[parser.index].type, parser.tokens[parser.index].value))

def parseImport(parser):
    i = Module.Import()
    parser.expect("import")
    i.module = parser.parseModuleName()
    if parser.peek("as"):
        parser.expect("as")
        i.alias = parser.parseTypeName()
    return i

def parseModule(parser):
    parser.expect("module")
    name = parser.parseModuleName()
    m = Module.Module()
    m.name = name
    parser.expect("leftcurly")
    while not parser.peek("rightcurly"):
        item = parseItem(parser, name)
        if item:
            m.items.append(item)
    parser.expect("rightcurly")
    return m
