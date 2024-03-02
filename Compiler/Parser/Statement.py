import Compiler.Syntax.Statement as Statement
import Compiler.Parser.Expr as Expr

def parseStatement(parser):
    if parser.peek("let"):
        parser.expect("let")
        let_s = Statement.LetStatement()
        let_s.var_name = parser.parseName()
        parser.expect("equal")
        let_s.rhs = Expr.parseExpr(parser)
        parser.expect("semicolon")
        return let_s
    elif parser.peek("leftcurly"):
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
    else:
        expr = Expr.parseExpr(parser)
        s = Statement.ExprStatement()
        s.requires_semicolon = True
        s.has_semicolon = parser.maybeParseSemicolon()
        s.expr = expr
        return s
