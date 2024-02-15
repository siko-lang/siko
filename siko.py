#!/bin/python3

import sys
import Parser
import Syntax
import NameResolver
import Typechecker
import IR
import Borrowchecker
import DataFlowPath
import Equality
import ForbiddenBorrows

def compile():
    program = Syntax.Program()

    for f in sys.argv[1:]:
        #print("Parsing ", f)
        parser = Parser.Parser()
        parser.parse(program, f)

    IR.convertProgram(program)

    resolver = NameResolver.Resolver()
    ir_program = resolver.resolve(program)

    Typechecker.checkProgram(ir_program)

    Borrowchecker.processProgram(ir_program)

    Equality.infer(ir_program)
    DataFlowPath.infer(ir_program)
    ForbiddenBorrows.infer(ir_program)

compile()