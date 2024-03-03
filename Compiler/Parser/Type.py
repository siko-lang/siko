import Compiler.Syntax.Type as SyntaxType

def parseType(parser):
    if parser.peek("typeid"):
        name = parser.parseQualifiedName()
        args = []
        if parser.peek("leftbracket"):
            parser.expect("leftbracket")
            while True:
                args.append(parseType(parser))
                if parser.peek("rightbracket"):
                    break
                else:
                    parser.expect("comma")
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
    elif parser.peek("fn"):
        parser.expect("fn")
        parser.expect("leftparen")
        params = []
        while True:
            param = parseType(parser)
            params.append(param)
            if parser.peek("rightparen"):
                break
            else:
                parser.expect("comma")
        parser.expect("rightparen")
        parser.expect("rightarrow")
        result = parseType(parser)
        kind = SyntaxType.Function(params, result)
        ty = SyntaxType.Type(kind)
    else:
        parser.expect("<type>")
    return ty

def parseGenericDeclaration(parser):
    decl = SyntaxType.GenericDeclaration()
    parser.expect("leftbracket")
    while True:
        name = parser.parseTypeName()
        vardecl = SyntaxType.GenericVarDeclaration(name)
        if parser.peek("colon"):
            parser.expect("colon")
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
        if parser.peek("comma"):
            parser.expect("comma")
        else:
            break
    if parser.peek("rightdoublearrow"):
        parser.expect("rightdoublearrow")
        constraints = []
        while True:
            constraint = parseType(parser)
            constraints.append(constraint)
            if parser.peek("comma"):
                parser.expect("comma")
            else:
                break
    parser.expect("rightbracket")
    return decl