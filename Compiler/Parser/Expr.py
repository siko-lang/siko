import Compiler.Syntax.Expr as Expr
import Compiler.Parser.Function as Function

def parseFunctionArgs(parser):
    parser.expect("leftparen")
    args = []
    while not parser.peek("rightparen"):
        args.append(parseExpr(parser))
        if parser.peek("rightparen"):
            break
        else:
            parser.expect("comma")
    parser.expect("rightparen")
    return args

def parseFunctionCall(parser):
    receiver = parsePrimary(parser)
    while True:
        if parser.peek("leftparen"):
            call = Expr.FunctionCall()
            call.id = receiver
            call.args = parseFunctionArgs(parser)
            receiver = call
        elif parser.peek("dot"):
            parser.expect("dot")
            name = parser.parseName()
            if parser.peek("leftparen"):
                args = parser.parseFunctionArgs()
                m = Expr.MemberCall()
                m.receiver = receiver
                m.name = name
                m.args = args
                receiver = m
            else:
                m = Expr.MemberAccess()
                m.receiver = receiver
                m.name = name
                receiver = m
        else:
            break
    return receiver

def parseIf(parser):
    parser.expect("if")
    if_expr = Expr.If()
    if_expr.cond = parseExpr(parser)
    if_expr.true_branch = Function.parseBlock(parser)
    if parser.peek("else"):
        parser.expect("else")
        if_expr.false_branch = Function.parseBlock(parser)
    return if_expr

def parseLoop(parser):
    parser.expect("loop")
    var = parser.parseName()
    parser.expect("equal")
    init = parser.parseExpr()
    body = parser.parseBlock()
    loop_expr = Expr.Loop()
    loop_expr.var = var
    loop_expr.init = init
    loop_expr.body = body
    return loop_expr

def parsePrimary(parser):
    if parser.peek("typeid"):
        name = parser.parseQualifiedName()
        r = Expr.TypeRef()
        r.name = name
        return r
    elif parser.peek("varid"):
        name = parser.parseName()
        r = Expr.VarRef()
        r.name = name
        return r
    elif parser.peek("leftcurly"):
        e = Function.parseBlock(parser)
        return e
    elif parser.peek("break"):
        parser.expect("break")
        e = Expr.Break()
        e.arg = parser.parseExpr()
        return e
    elif parser.peek("continue"):
        parser.expect("continue")
        e = Expr.Continue()
        e.arg = parser.parseExpr()
        return e
    elif parser.peek("return"):
        parser.expect("return")
        e = Expr.Return()
        e.arg = parser.parseExpr()
        return e
    elif parser.peek("loop"):
        return parser.parseLoop()
    elif parser.peek("if"):
        return parser.parseIf()
    elif parser.peek("true"):
        parser.expect("true")
        e = Expr.BoolLiteral()
        e.value = True
        return e
    elif parser.peek("false"):
        parser.expect("false")
        e = Expr.BoolLiteral()
        e.value = False
        return e
    else:
        parser.error("expected expr, found %s" % parser.tokens[parser.index].type)

def parseExpr(parser):
    return parseFunctionCall(parser)
