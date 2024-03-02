import Compiler.Parser.Expr as Expr
import Compiler.Syntax.Type as SyntaxType
import Compiler.Parser.Type as Type
import Compiler.Syntax.Function as Function
import Compiler.Parser.Statement as Statement
import Compiler.Syntax.Statement as SyntaxStatement

def parseFunction(parser, module_name):
    fn = Function.Function()
    fn.module_name = module_name
    parser.expect("fn")
    name = parser.parseName()
    fn.name = name
    parser.expect("leftparen")
    while not parser.peek("rightparen"):
        arg = parseArgDef(parser)    
        fn.args.append(arg)    
        if not parser.peek("rightparen"):
            parser.expect("comma")
    parser.expect("rightparen")
    if parser.peek("rightarrow"):
        parser.expect("rightarrow")
        fn.return_type = Type.parseType(parser)
    else:
        empty_tuple = SyntaxType.Type(SyntaxType.Tuple([]))
        fn.return_type = empty_tuple
    if parser.peek("equal"):
        parser.expect("equal")
        parser.expect("extern")
    else:
        fn.body = parseBlock(parser)
    return fn

def parseArgDef(parser):
    name = parser.parseName()
    parser.expect("colon")
    ty = Type.parseType(parser)
    arg = Function.Arg()
    arg.name = name
    arg.type = ty
    return arg

def parseBlock(parser):
    parser.expect("leftcurly")
    block = Function.Block()
    while not parser.peek("rightcurly"):
        s = Statement.parseStatement(parser)
        block.statements.append(s)
        if parser.peek("rightcurly"):
            break
        else:
            if isinstance(s, SyntaxStatement.ExprStatement):
                if s.requires_semicolon and not s.has_semicolon:
                    parser.error("Non trailing expr requires semicolon!")
    parser.expect("rightcurly")
    return block
