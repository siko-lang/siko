import Compiler.Syntax.Trait as Trait
import Compiler.Parser.Function as Function

def parseTrait(parser, module_name):
    parser.expect("trait")
    trait = Trait.Trait()
    trait.name = parser.parseTypeName()
    if parser.peek("leftbracket"):
        parser.expect("leftbracket")
        dependentParams = False
        while True:
            if parser.peek("greaterthan"):
                parser.expect("greaterthan")
                dependentParams = True
                break
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
                    trait.declarations.append(fn)
                else:
                    trait.methods.append(fn)
            else:
                break
        parser.expect("rightcurly")
    return trait
