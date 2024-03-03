import Compiler.Syntax.Expr as Expr
import Compiler.Parser.Function as Function
import Compiler.Parser.Pattern as Pattern
import Compiler.Token as Token

def parseFunctionArgs(parser):
    parser.expect(Token.LeftParen())
    args = []
    while not parser.peek(Token.RightParen()):
        args.append(parseExpr(parser))
        if parser.peek(Token.RightParen()):
            break
        else:
            parser.expect(Token.Comma())
    parser.expect(Token.RightParen())
    return args

def parseFunctionCall(parser):
    receiver = parsePrimary(parser)
    while True:
        if parser.peek(Token.LeftParen()):
            call = Expr.FunctionCall()
            call.id = receiver
            call.args = parseFunctionArgs(parser)
            receiver = call
        elif parser.peek(Token.Dot()):
            parser.expect(Token.Dot())
            name = parser.parseName()
            if parser.peek(Token.LeftParen()):
                args = parseFunctionArgs(parser)
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

def parseMatch(parser):
    parser.expect("match")
    match_expr = Expr.Match()
    match_expr.body = parseExpr(parser)
    parser.expect(Token.LeftCurly())
    while True:
        branch = Expr.MatchBranch()
        match_expr.branches.append(branch)
        branch.pattern = Pattern.parsePattern(parser)
        parser.expect(Token.RightArrow())
        branch.body = parseExpr(parser)
        if parser.peek(Token.Comma()):
            parser.expect(Token.Comma())
        if parser.peek(Token.RightCurly()):
            break
    parser.expect(Token.RightCurly())
    return match_expr

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

def parseForLoop(parser):
    parser.expect("for")
    pattern = Pattern.parsePattern(parser)
    parser.expect("in")
    init = parseExpr(parser)
    body = Function.parseBlock(parser)
    loop_expr = Expr.ForLoop()
    loop_expr.pattern = pattern
    loop_expr.init = init
    loop_expr.body = body
    return loop_expr

def parsePrimary(parser):
    if parser.peek("string"):
        l = Expr.StringLiteral()
        l.value = parser.tokens[parser.index].value
        parser.step()
        return l
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
    elif parser.peek(Token.LeftCurly()):
        e = Function.parseBlock(parser)
        return e
    elif parser.peek("break"):
        parser.expect("break")
        e = Expr.Break()
        e.arg = parseExpr(parser)
        return e
    elif parser.peek("continue"):
        parser.expect("continue")
        e = Expr.Continue()
        if not parser.peek(Token.Semicolon()) and not parser.peek(Token.Comma()):
            e.arg = parseExpr(parser)
        return e
    elif parser.peek("return"):
        parser.expect("return")
        e = Expr.Return()
        e.arg = parseExpr(parser)
        return e
    elif parser.peek("loop"):
        return parseLoop(parser)
    elif parser.peek("match"):
        return parseMatch(parser)
    elif parser.peek("for"):
        return parseForLoop(parser)
    elif parser.peek("if"):
        return parser.parseIf()
    elif parser.peek(Token.LeftParen()):
        parser.expect(Token.LeftParen())
        items = []
        if not parser.peek(Token.RightParen()):
            while True:
                item = parseExpr(parser)
                items.append(item)
                if parser.peek(Token.Comma()):
                    parser.expect(Token.Comma())
                else:
                    break
        parser.expect(Token.RightParen())
        if len(items) == 1:
            return items[0]
        e = Expr.Tuple()
        e.args = items
        return e
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

def opsTable():
    ops_table = [[Token.And(), Token.Or()],
                  [Token.DoubleEqual(), Token.NotEqual()],
                  [Token.LessThan(),
                   Token.GreaterThan(),
                   Token.LessThanOrEqual(),
                   Token.GreaterThanOrEqual()],
                  [Token.Plus(), Token.Minus()],
                  [Token.Mul(), Token.Div()]]
    return ops_table

def callNext(parser, index):
    if index + 1 < len(opsTable()):
        return parseBinary(parser, index + 1)
    else:
        return parseFunctionCall(parser)

def parseBinary(parser, index):
    expr = callNext(parser, index)
    while True:
        ops = opsTable()[index]
        matchingOp = None
        for op in ops:
            if parser.peek(op):
                matchingOp = op
                break
        if matchingOp is not None:
            parser.expect(op)
            rhs = callNext(parser, index)
            e = Expr.BinaryOp()
            e.op = op
            e.rhs = rhs
            e.lhs = expr
            expr = e
        else:
            break
    return expr

def parseExpr(parser):
    return parseBinary(parser, 0)
