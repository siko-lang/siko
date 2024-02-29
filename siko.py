#!/bin/python3

import sys
import Compiler.Parser as Parser
import Compiler.Syntax as Syntax
import Compiler.NameResolver as NameResolver
import Compiler.Typechecker as Typechecker
import Compiler.IR.Builder as Builder
import Compiler.Ownership.Borrowchecker as Borrowchecker
import Compiler.Ownership.DataFlowPath as DataFlowPath
import Compiler.Ownership.ForbiddenBorrows as ForbiddenBorrows
import Compiler.Ownership.Monomorphizer as Monomorphizer
import Compiler.Transpiler as Transpiler
import Compiler.Ownership.DataFlowProfileInference as DataFlowProfileInference

def compile():
    program = Syntax.Program()
    args = sys.argv[1:]
    while True:
        name = args.pop(0)
        if name=="-o":
            break
        parser = Parser.Parser()
        parser.parse(program, name)

    output = args.pop()

    Builder.convertProgram(program)

    resolver = NameResolver.Resolver()
    ir_program = resolver.resolve(program)

    Typechecker.checkProgram(ir_program)

    Borrowchecker.processProgram(ir_program)

    #print("Building data flow profiles")

    profile_store = DataFlowProfileInference.infer(ir_program)

    #print("Building data flow profiles done")

    (classes, functions) = Monomorphizer.monomorphize(ir_program, profile_store)
    Transpiler.transpile(classes, functions, output)

compile()