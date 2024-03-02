import Compiler.Syntax.Type as SyntaxType

def parseConstraints(parser):
    parser.expect("leftbracket")
    type_arg = parser.parseName()
    parser.expect("rightbracket")

def parseType(parser):
    if parser.peek("typeid"):
        name = parser.parseQualifiedName()
        args = []
        if parser.peek("leftbracket"):
            parser.expect("leftbracket")
            args.append(parseType(parser))
            parser.expect("rightbracket")
        kind = SyntaxType.Named(name, args)
        ty = SyntaxType.Type(kind)
    elif parser.peek("leftparen"):
        parser.expect("leftparen")
        items = []
        while True:
            item = parseType(parser)
            items.append(item)
            if parser.peek("rightparen"):
                break
            else:
                parser.expect("comma")
        parser.expect("rightparen")
        kind = SyntaxType.Tuple(items)
        ty = SyntaxType.Type(kind)
    return ty

def parseGenericDeclaration(parser):
    decl = SyntaxType.GenericDeclaration()
    parser.expect("leftbracket")
    while True:
        name = parser.parseTypeName()
        vardecl = SyntaxType.GenericVarDeclaration(name)
        decl.generics.append(vardecl)
        if parser.peek("rightbracket"):
            break
        else:
            parser.expect("comma")
    parser.expect("rightbracket")
    return decl