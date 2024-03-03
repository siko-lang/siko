import Compiler.Parser.Expr as Expr
import Compiler.Syntax.Type as SyntaxType
import Compiler.Parser.Type as Type
import Compiler.Syntax.Function as Function
import Compiler.Parser.Statement as Statement
import Compiler.Syntax.Statement as SyntaxStatement
import Compiler.Token as Token

def parseFunction(parser, module_name):
    fn = Function.Function()
    fn.module_name = module_name
    parser.expect("fn")
    name = parser.parseName()
    fn.name = name
    if parser.peek(Token.LeftBracket()):
        fn.generics = Type.parseGenericDeclaration(parser)
    parser.expect(Token.LeftParen())
    while not parser.peek(Token.RightParen()):
        arg = parseParamDef(parser)    
        fn.params.append(arg)    
        if not parser.peek(Token.RightParen()):
            parser.expect(Token.Comma())
    parser.expect(Token.RightParen())
    if parser.peek(Token.RightArrow()):
        parser.expect(Token.RightArrow())
        fn.return_type = Type.parseType(parser)
    else:
        empty_tuple = SyntaxType.Type(SyntaxType.Tuple([]))
        fn.return_type = empty_tuple
    if parser.peek("equal"):
        parser.expect("equal")
        parser.expect("extern")
    else:
        if parser.peek(Token.LeftCurly()):
            fn.body = parseBlock(parser)
    return fn

def parseParamDef(parser):
    arg = Function.Param()
    if parser.peek("mut"):
        parser.expect("mut")
        arg.mutable = True
    name = parser.parseName()
    if name != "self":
        parser.expect(Token.Colon())
        ty = Type.parseType(parser)
    else:
        ty = None
    arg.name = name
    arg.type = ty
    return arg

def parseBlock(parser):
    parser.expect(Token.LeftCurly())
    block = Function.Block()
    while not parser.peek(Token.RightCurly()):
        s = Statement.parseStatement(parser)
        block.statements.append(s)
        if parser.peek(Token.RightCurly()):
            break
        else:
            if isinstance(s, SyntaxStatement.ExprStatement):
                if s.requires_semicolon and not s.has_semicolon:
                    parser.error("Non trailing expr requires semicolon!")
    parser.expect(Token.RightCurly())
    return block
