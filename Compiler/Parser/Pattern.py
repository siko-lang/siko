import Compiler.Syntax.Pattern as Pattern
import Compiler.Token as Token

def parsePattern(parser):
    if parser.peek(Token.Wildcard()):
        parser.expect(Token.Wildcard())
        return Pattern.Wildcard()
    elif parser.peek("typeid"):
        p = Pattern.Named()
        p.name = parser.parseTypeName()
        if parser.peek(Token.LeftParen()):
            parser.expect(Token.LeftParen())
            while True:
                arg = parsePattern(parser)
                p.args.append(arg)
                if parser.peek(Token.Comma()):
                    parser.expect(Token.Comma())
                else:
                    break
            parser.expect(Token.RightParen())
        return p
    elif parser.peek(Token.LeftParen()):
        p = Pattern.Tuple()
        if parser.peek(Token.LeftParen()):
            parser.expect(Token.LeftParen())
            while True:
                arg = parsePattern(parser)
                p.args.append(arg)
                if parser.peek(Token.Comma()):
                    parser.expect(Token.Comma())
                else:
                    break
            parser.expect(Token.RightParen())
        return p
    elif parser.peek("mut"):
        parser.step()
        p = Pattern.Bind()
        p.name = parser.parseName()
        p.mutable = True
        return p
    elif parser.peek("varid"):
        p = Pattern.Bind()
        p.name = parser.parseName()
        return p
    else:
        parser.expect("<pattern>")
        