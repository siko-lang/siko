#!/bin/python3

import sys
import Parser
import Syntax
import NameResolver

def compile():
    program = Syntax.Program()

    for f in sys.argv[1:]:
        print("Parsing ", f)
        parser = Parser.Parser()
        parser.parse(program, f)

    resolver = NameResolver.Resolver()
    resolver.resolve(program)

class Processor(object):
    def __init__(self):
        self.instructions = []

    def addInstruction(self, instruction):
        index = len(self.instructions)
        self.instructions.append(instruction)
        return index

    def processArgs(self, eargs):
        args = []
        for arg in eargs:
            args.append(self.processExpr(arg))
        args = map(lambda x: str(x), args)
        return args

    def processExpr(self, expr):
        if isinstance(expr, Syntax.Block):
            self.addInstruction("<block begin>")
            last = None
            for s in expr.statements:
                last = self.processExpr(s)
            self.addInstruction("<block end>")
            return last
        elif isinstance(expr, Syntax.LetStatement):
            id = self.processExpr(expr.rhs)
            return self.addInstruction("Let %s = $%s" % (expr.var_name, id))
        elif isinstance(expr, Syntax.ExprStatement):
            return self.processExpr(expr.expr)
        elif isinstance(expr, Syntax.MemberCall):
            id = self.processExpr(expr.receiver)
            args = self.processArgs(expr.args)
            return self.addInstruction("$%s.%s(%s)" % (id, expr.name, ", ".join(args)))
        elif isinstance(expr, Syntax.FunctionCall):
            args = self.processArgs(expr.args)
            if isinstance(expr.id, Syntax.VarRef):
                return self.addInstruction("%s(%s)" % (expr.id.name, ", ".join(args)))
            else:
                id = self.processExpr(expr.id)
                return self.addInstruction("$%s()" % id)
        elif isinstance(expr, Syntax.MemberAccess):
            id = self.processExpr(expr.receiver)
            return self.addInstruction("$%s.%s" % (id, expr.name))
        elif isinstance(expr, Syntax.VarRef):
            return self.addInstruction("%s" % expr.name)
        elif isinstance(expr, Syntax.If):
            cond = self.processExpr(expr.cond)
            true_branch = self.processExpr(expr.true_branch)
            if expr.false_branch:
                false_branch = self.processExpr(expr.false_branch)
                return self.addInstruction("if $%s then $%s else $%s" % (cond, true_branch, false_branch))
            else:
                return self.addInstruction("if $%s then $%s" % (cond, true_branch))
        elif isinstance(expr, Syntax.Loop):
            init = self.processExpr(expr.init)
            body = self.processExpr(expr.body)
            return self.addInstruction("loop %s = $%s $%s" % (expr.var, init, body))
        elif isinstance(expr, Syntax.Break):
            arg = self.processExpr(expr.arg)
            return self.addInstruction("break $%s" % arg)
        elif isinstance(expr, Syntax.Continue):
            arg = self.processExpr(expr.arg)
            return self.addInstruction("continue $%s" % arg)
        elif isinstance(expr, Syntax.Return):
            arg = self.processExpr(expr.arg)
            return self.addInstruction("return $%s" % arg)
        else:
            print("Expr not handled", type(expr))

# for m in program.modules:
#     print("Processing module %s" % m.name)
#     for item in m.items:
#         if isinstance(item, Syntax.Function):
#             fn = item
#             print("Processing fn %s" % fn.name)
#             processor = Processor()
#             processor.processExpr(fn.body)
#             for (index, i) in enumerate(processor.instructions):
#                 print("$%d. %s" % (index, i))
            
compile()