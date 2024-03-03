import Compiler.Syntax.Statement as Statement
import Compiler.Parser.Expr as Expr
import Compiler.Token as Token

def parseStatement(parser):
    if parser.peek("let"):
        parser.expect("let")
        let_s = Statement.LetStatement()
        let_s.var_name = parser.parseName()
        parser.expect("equal")
        let_s.rhs = Expr.parseExpr(parser)
        parser.expect(Token.Semicolon())
        return let_s
    elif parser.peek(Token.LeftCurly()):
        expr = parser.parseBlock()
        s = Statement.ExprStatement()
        s.requires_semicolon = False
        s.has_semicolon = parser.maybeParseSemicolon()
        s.expr = expr
        return s
    elif parser.peek("if"):
        expr = Expr.parseIf(parser)
        s = Statement.ExprStatement()
        s.requires_semicolon = False
        s.has_semicolon = parser.maybeParseSemicolon()
        s.expr = expr
        return s
    elif parser.peek("loop"):
        expr = Expr.parseLoop(parser)
        s = Statement.ExprStatement()
        s.requires_semicolon = False
        s.has_semicolon = parser.maybeParseSemicolon()
        s.expr = expr
        return s
    elif parser.peek("for"):
        expr = Expr.parseForLoop(parser)
        s = Statement.ExprStatement()
        s.requires_semicolon = False
        s.has_semicolon = parser.maybeParseSemicolon()
        s.expr = expr
        return s
    elif parser.peek("match"):
        expr = Expr.parseMatch(parser)
        s = Statement.ExprStatement()
        s.requires_semicolon = False
        s.has_semicolon = parser.maybeParseSemicolon()
        s.expr = expr
        return s
    else:
        lhs = Expr.parseExpr(parser)
        if parser.peek("equal"):
            parser.expect("equal")
            rhs = Expr.parseExpr(parser)
            parser.expect(Token.Semicolon())
            s = Statement.AssignStatement()
            s.lhs = lhs
            s.rhs = rhs
            return s
        else:
            s = Statement.ExprStatement()
            s.requires_semicolon = True
            s.has_semicolon = parser.maybeParseSemicolon()
            s.expr = lhs
            return s
