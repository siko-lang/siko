import Compiler.Syntax.Type as SyntaxType
import Compiler.Token as Token

def parseType(parser):
    if parser.peek("typeid"):
        name = parser.parseQualifiedName()
        args = []
        if parser.peek(Token.LeftBracket()):
            parser.expect(Token.LeftBracket())
            while True:
                args.append(parseType(parser))
                if parser.peek(Token.RightBracket()):
                    break
                else:
                    parser.expect(Token.Comma())
            parser.expect(Token.RightBracket())
        kind = SyntaxType.Named(name, args)
        ty = SyntaxType.Type(kind)
    elif parser.peek(Token.LeftParen()):
        parser.expect(Token.LeftParen())
        items = []
        while True:
            item = parseType(parser)
            items.append(item)
            if parser.peek(Token.RightParen()):
                break
            else:
                parser.expect(Token.Comma())
        parser.expect(Token.RightParen())
        kind = SyntaxType.Tuple(items)
        ty = SyntaxType.Type(kind)
    elif parser.peek("fn"):
        parser.expect("fn")
        parser.expect(Token.LeftParen())
        params = []
        while True:
            param = parseType(parser)
            params.append(param)
            if parser.peek(Token.RightParen()):
                break
            else:
                parser.expect(Token.Comma())
        parser.expect(Token.RightParen())
        parser.expect(Token.RightArrow())
        result = parseType(parser)
        kind = SyntaxType.Function(params, result)
        ty = SyntaxType.Type(kind)
    else:
        parser.expect("<type>")
    return ty

def parseGenericDeclaration(parser):
    decl = SyntaxType.GenericDeclaration()
    parser.expect(Token.LeftBracket())
    while True:
        name = parser.parseTypeName()
        vardecl = SyntaxType.GenericVarDeclaration(name)
        if parser.peek(Token.Colon()):
            parser.expect(Token.Colon())
            deps = []
            while True:
                dep = parser.parseTypeName()
                deps.append(dep)
                if parser.peek("plus"):
                    parser.expect("plus")
                else:
                    break
            vardecl.deps = deps
        decl.generics.append(vardecl)
        if parser.peek(Token.Comma()):
            parser.expect(Token.Comma())
        else:
            break
    if parser.peek(Token.RightDoubleArrow()):
        parser.expect(Token.RightDoubleArrow())
        constraints = []
        while True:
            constraint = parseType(parser)
            constraints.append(constraint)
            if parser.peek(Token.Comma()):
                parser.expect(Token.Comma())
            else:
                break
    parser.expect(Token.RightBracket())
    return decl