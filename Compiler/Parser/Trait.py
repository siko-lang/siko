import Compiler.Syntax.Trait as Trait
import Compiler.Parser.Function as Function
import Compiler.Parser.Type as Type

def parseTrait(parser, module_name):
    parser.expect("trait")
    trait = Trait.Trait()
    if parser.peek("leftbracket"):
        trait.generics = Type.parseGenericDeclaration(parser)
    trait.name = parser.parseTypeName()
    parser.expect("leftbracket")
    dependentParams = False
    while True:
        param = parser.parseTypeName()
        trait.generic_parameters.append(param)
        if parser.peek("rightbracket"):
            break
        if parser.peek("greaterthan"):
            parser.expect("greaterthan")
            dependentParams = True
            break
        parser.expect("comma")
    if dependentParams:
        while True:
            param = parser.parseTypeName()
            trait.dependent_parameters.append(param)
            if parser.peek("rightbracket"):
                break
    parser.expect("rightbracket")
    if parser.peek("leftcurly"):
        parser.expect("leftcurly")
        while True:
            if parser.peek("fn"):
                fn = Function.parseFunction(parser, module_name)
                if fn.body is None:
                    trait.method_declarations.append(fn)
                else:
                    trait.methods.append(fn)
            else:
                break
        parser.expect("rightcurly")
    return trait

def parseInstance(parser, module_name):
    parser.expect("instance")
    instance = Trait.Instance()
    if parser.peek("leftbracket"):
        instance.generics = Type.parseGenericDeclaration(parser)
    instance.type = Type.parseType(parser)
    if parser.peek("leftcurly"):
        parser.expect("leftcurly")
        while True:
            if parser.peek("fn"):
                fn = Function.parseFunction(parser, module_name)
                if fn.body is None:
                    instance.declarations.append(fn)
                else:
                    instance.methods.append(fn)
            else:
                break
        parser.expect("rightcurly")
    return instance
