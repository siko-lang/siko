#!/bin/python3

import sys
import Parser
import Syntax
import NameResolver
import Typechecker
import IR

def compile():
    program = Syntax.Program()

    for f in sys.argv[1:]:
        #print("Parsing ", f)
        parser = Parser.Parser()
        parser.parse(program, f)

    IR.convertProgram(program)

    resolver = NameResolver.Resolver()
    resolver.resolve(program)

    Typechecker.checkProgram(program)

compile()