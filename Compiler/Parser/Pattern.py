import Compiler.Syntax.Pattern as Pattern

def parsePattern(parser):
    if parser.peek("wildcard"):
        parser.expect("wildcard")
        return Pattern.Wildcard()
    elif parser.peek("typeid"):
        p = Pattern.Named()
        p.name = parser.parseTypeName()
        if parser.peek("leftparen"):
            parser.expect("leftparen")
            while True:
                arg = parsePattern(parser)
                p.args.append(arg)
                if parser.peek("comma"):
                    parser.expect("comma")
                else:
                    break
            parser.expect("rightparen")
        return p
    elif parser.peek("leftparen"):
        p = Pattern.Tuple()
        if parser.peek("leftparen"):
            parser.expect("leftparen")
            while True:
                arg = parsePattern(parser)
                p.args.append(arg)
                if parser.peek("comma"):
                    parser.expect("comma")
                else:
                    break
            parser.expect("rightparen")
        return p
    else:
        parser.expect("<pattern>")
        