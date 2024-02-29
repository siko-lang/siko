import Compiler.Syntax as Syntax
import Compiler.Util as Util
import Compiler.IR.Instruction as Instruction
import Compiler.IR.IR as IR

class Builder(object):
    def __init__(self):
        self.blocks = []
        self.current = []

    def currentBlock(self):
        return self.current[-1]        

    def addInstruction(self, instruction):
        return self.currentBlock().addInstruction(instruction)

    def processArgs(self, eargs):
        args = []
        for arg in eargs:
            args.append(self.processExpr(arg))
        return args

    def createBlock(self):
        block = IR.Block()
        block.id = len(self.blocks)
        self.blocks.append(block)
        self.current.append(block)
        return block

    def processBlock(self, expr, rootBlock=False):
        if rootBlock:
            block = self.currentBlock()
        else:
            block = self.createBlock()
        last = None
        lastStatement = None
        for s in expr.statements:
            lastStatement = s
            last = self.processExpr(s)
        if lastStatement:
            if isinstance(lastStatement, Syntax.ExprStatement):
                if lastStatement.has_semicolon:
                    unit = Instruction.NamedFunctionCall()
                    unit.name = Util.getUnit()
                    self.addInstruction(unit)
            else:
                unit = Instruction.NamedFunctionCall()
                unit.name = Util.getUnit()
                self.addInstruction(unit)
        else:
            unit = Instruction.NamedFunctionCall()
            unit.name = Util.getUnit()
            self.addInstruction(unit)
        self.current.pop()
        return block.id

    def processExpr(self, expr, rootBlock = False, packBlock=True):
        if isinstance(expr, Syntax.Block):
            id = self.processBlock(expr, rootBlock)
            if packBlock:
                blockref = Instruction.BlockRef()
                blockref.value = id
                return self.addInstruction(blockref)
            return id
        elif isinstance(expr, Syntax.LetStatement):
            id = self.processExpr(expr.rhs)
            bind = Instruction.Bind()
            bind.name = expr.var_name
            bind.rhs = id
            return self.addInstruction(bind)
        elif isinstance(expr, Syntax.ExprStatement):
            return self.processExpr(expr.expr)
        elif isinstance(expr, Syntax.MemberCall):
            receiver = self.processExpr(expr.receiver)
            args = self.processArgs(expr.args)
            call = Instruction.MethodCall()
            call.receiver = receiver
            call.name = expr.name
            call.args = args
            return self.addInstruction(call)
        elif isinstance(expr, Syntax.FunctionCall):
            args = self.processArgs(expr.args)
            if isinstance(expr.id, Syntax.VarRef):
                call = Instruction.NamedFunctionCall()
                call.name = expr.id.name
                call.args = args
                return self.addInstruction(call)
            elif isinstance(expr.id, Syntax.TypeRef):
                call = Instruction.NamedFunctionCall()
                call.name = expr.id.name
                call.args = args
                return self.addInstruction(call)
            else:
                id = self.processExpr(expr.id)
                call = Instruction.DynamicFunctionCall()
                call.callable = id
                call.args = args
                return self.addInstruction(call)
        elif isinstance(expr, Syntax.MemberAccess):
            isValueRef = True
            fields = [expr.name]
            var = None
            current = expr
            while True:
                if isinstance(current.receiver, Syntax.VarRef):
                    var = current.receiver.name
                    break
                if isinstance(current.receiver, Syntax.MemberAccess):
                    fields.append(current.receiver.name)
                    current = current.receiver
                    continue
                isValueRef = False
                break
            if isValueRef:
                fields.reverse()
                value_ref= Instruction.ValueRef()
                value_ref.name = var
                value_ref.fields = fields
                return self.addInstruction(value_ref)
            else:
                receiver = self.processExpr(expr.receiver)
                access = Instruction.MemberAccess()
                access.receiver = receiver
                access.name = expr.name
                return self.addInstruction(access)
        elif isinstance(expr, Syntax.VarRef):
            ref = Instruction.ValueRef()
            ref.name = expr.name
            ref.fields = []
            return self.addInstruction(ref)
        elif isinstance(expr, Syntax.If):
            if_instr = Instruction.If()
            if_instr.cond = self.processExpr(expr.cond)
            if_instr.true_branch = self.processExpr(expr.true_branch, rootBlock=False, packBlock=False)
            if expr.false_branch:
                if_instr.false_branch = self.processExpr(expr.false_branch, rootBlock=False, packBlock=False)
            return self.addInstruction(if_instr)
        elif isinstance(expr, Syntax.Loop):
            init = self.processExpr(expr.init)
            body = self.processExpr(expr.body, rootBlock=False)
            loop = Instruction.Loop()
            loop.var = expr.var
            loop.init = init
            loop.body = body
            return self.addInstruction(loop)
        elif isinstance(expr, Syntax.Break):
            arg = self.processExpr(expr.arg)
            br = Instruction.Break()
            br.arg = arg
            return self.addInstruction(br)
        elif isinstance(expr, Syntax.Continue):
            arg = self.processExpr(expr.arg)
            cont = Instruction.Continue()
            cont.arg = arg
            return self.addInstruction(cont)
        elif isinstance(expr, Syntax.Return):
            arg = self.processExpr(expr.arg)
            ret = Instruction.Return()
            ret.arg = arg
            return self.addInstruction(ret)
        elif isinstance(expr, Syntax.BoolLiteral):
            b = Instruction.BoolLiteral()
            b.value = expr.value
            return self.addInstruction(b)
        else:
            print("Expr not handled", type(expr))

def convertProgram(program):
    for m in program.modules:
        #print("Processing module %s" % m.name)
        for item in m.items:
            if isinstance(item, Syntax.Function):
                fn = item
                #print("Processing fn %s" % fn.name)
                processor = Builder()
                block = processor.createBlock()
                for (index, arg) in enumerate(fn.args):
                    arg_name = "arg_%s" % index
                    arg_ref = Instruction.ValueRef()
                    arg_ref.name = arg_name
                    arg_ref.fields = []
                    arg_ref_id = block.addInstruction(arg_ref)
                    arg_bind = Instruction.Bind()
                    arg_bind.name = arg.name
                    arg.name = arg_name
                    arg_bind.rhs = arg_ref_id
                    block.addInstruction(arg_bind)
                processor.processExpr(fn.body, rootBlock=True, packBlock=False)
                body = IR.Body()
                body.blocks = processor.blocks
                fn.body = body
                #fn.body.dump()
            if isinstance(item, Syntax.Class):
                for m in item.methods:
                    #print("Processing method %s" % m.name)
                    processor = Builder()
                    block = processor.createBlock()
                    for (index, arg) in enumerate(m.args):
                        arg_name = "arg_%s" % index
                        arg_ref = Instruction.ValueRef()
                        arg_ref.name = arg_name
                        arg_ref.fields = []
                        arg_ref_id = block.addInstruction(arg_ref)
                        arg_bind = Instruction.Bind()
                        arg_bind.name = arg.name
                        arg.name = arg_name
                        arg_bind.rhs = arg_ref_id
                        block.addInstruction(arg_bind)
                    processor.processExpr(m.body, rootBlock=True, packBlock=False)
                    body = Instruction.Body()
                    body.blocks = processor.blocks
                    m.body = body
                

                