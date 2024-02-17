import Syntax
import Util

class TempVar(object):
    def __init__(self):
        self.value = 0
        self.arg = False

    def __str__(self):
        if self.arg:
            return "$arg_%s" % self.value
        else:
            return "$tmp_%s" % self.value

    def __repr__(self):
        return self.__str__()

    def __eq__(self, other):
        if isinstance(other, TempVar):
            return self.value == other.value
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

class InstructionId(object):
    def __init__(self):
        self.block = 0
        self.value = 0

    def prev(self):
        p = InstructionId()
        p.block = self.block
        p.value = self.value - 1
        return p
    
    def __eq__(self, other):
        return self.block == other.block and self.value == other.value

    def __hash__(self):
        return self.block.__hash__()

    def __str__(self):
        return "$%s.%s" % (self.block, self.value)

    def __repr__(self) -> str:
        return self.__str__()

class BaseInstruction(object):
    def __init__(self):
        self.id = 0
        self.tv_info = None
        self.members = []

class BlockRef(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.value = 0

    def __str__(self):
        return "block ref: #%s" % self.value

class NamedFunctionCall(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.ctor = False
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        if self.ctor:
            return "%s(%s)/ctor" % (self.name, ", ".join(args))
        else:
            return "%s(%s)" % (self.name, ", ".join(args))

class DynamicFunctionCall(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.callable = None
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        return "%s(%s)" % (self.callable, ", ".join(args))

class MethodCall(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.receiver = None
        self.name = None
        self.args = []

    def __str__(self):
        args = map(lambda x: str(x), self.args)
        return "%s.%s(%s)" % (self.receiver, self.name, ", ".join(args))

class Bind(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.rhs = None

    def __str__(self):
        return "%s = %s" % (self.name, self.rhs)

class MemberAccess(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.receiver = None
        self.name = None
        self.index = 0
        
    def __str__(self):
        return "%s.%s" % (self.receiver, self.name)

class VarRef(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.bind_id = None
        self.borrow = False
        
    def __str__(self):
        return "%s/%s" % (self.name, self.bind_id)

class ValueRef(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.bind_id = None
        self.fields = []
        self.indices = []
        self.borrow = False

    def __str__(self):
        if len(self.fields) > 0:
            fields = ".".join(self.fields)
            return "%s.%s/%s" % (self.name, fields, self.bind_id)
        else:
            return "%s/%s" % (self.name, self.bind_id)

class DropVar(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.name = None
        self.cancelled = False
        
    def __str__(self):
        if self.cancelled:
            return "drop(%s)/Inactive" % (self.name)
        else:
            return "drop(%s)/Active" % (self.name)

class Nop(BaseInstruction):
    def __init__(self):
        super().__init__()

    def __str__(self):
        return "nop"

class If(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.cond = None
        self.true_branch = None
        self.false_branch = None

    def __str__(self):
        if self.false_branch:
            return "if %s then %s else %s" % (self.cond, self.true_branch, self.false_branch)
        else:
            return "if %s then %s" % (self.cond, self.true_branch)

class Loop(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.var = None
        self.init = None
        self.body = None

    def __str__(self):
        return "loop %s = %s %s" % (self.var, self.init, self.body)

class Break(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None

    def __str__(self):
        return "break %s" % self.arg

class Continue(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None
    
    def __str__(self):
        return "continue %s" % self.arg

class Return(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None
    
    def __str__(self):
        return "return %s" % self.arg

class Converter(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.arg = None
    
    def __str__(self):
        return "convert %s" % self.arg

class BoolLiteral(BaseInstruction):
    def __init__(self):
        super().__init__()
        self.value = None
    
    def __str__(self):
        return "bool %s" % self.value

class Body(object):
    def __init__(self):
        self.blocks = []

    def dump(self):
        for b in self.blocks:
           print("#%d. block:" % b.id)
           b.dump()

    def getFirst(self):
        for b in self.blocks:
            if b.id == 0:
                return b
        return None

    def getInstruction(self, id):
        return self.blocks[id.block].instructions[id.value]

    def getAllMembers(self):
        members = []
        for b in self.blocks:
            for i in b.instructions:
                for m in i.members:
                    members.append(m)
        return members

    def getBlock(self, blockref):
        value = blockref
        if isinstance(blockref, BlockRef):
            value = blockref.value
        for b in self.blocks:
            if b.id == value:
                return b
        return None

class Block(object):
    def __init__(self):
        self.id = None
        self.instructions = []

    def addInstruction(self, instruction):
        index = len(self.instructions)
        id = InstructionId()
        id.value = index
        id.block = self.id
        instruction.id = id
        self.instructions.append(instruction)
        return id

    def getLast(self):
        return self.instructions[-1]

    def getLastReal(self):
        last = None
        for i in self.instructions:
            if isinstance(i, DropVar):
                continue
            if isinstance(i, Nop):
                continue
            last = i
        return last

    def dump(self):
        for (index, i) in enumerate(self.instructions):
           print("$%d. %s" % (index, i))

class Program(object):
    def __init__(self):
        self.modules = []
        self.functions = {}
        self.classes = {}

class Processor(object):
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
        block = Block()
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
                    unit = NamedFunctionCall()
                    unit.name = str(Util.getUnit())
                    self.addInstruction(unit)
            else:
                unit = NamedFunctionCall()
                unit.name = str(Util.getUnit())
                self.addInstruction(unit)
        else:
            unit = NamedFunctionCall()
            unit.name = str(Util.getUnit())
            self.addInstruction(unit)
        self.current.pop()
        return block.id

    def processExpr(self, expr, rootBlock = False, packBlock=True):
        if isinstance(expr, Syntax.Block):
            id = self.processBlock(expr, rootBlock)
            if packBlock:
                blockref = BlockRef()
                blockref.value = id
                return self.addInstruction(blockref)
            return id
        elif isinstance(expr, Syntax.LetStatement):
            id = self.processExpr(expr.rhs)
            bind = Bind()
            bind.name = expr.var_name
            bind.rhs = id
            return self.addInstruction(bind)
        elif isinstance(expr, Syntax.ExprStatement):
            return self.processExpr(expr.expr)
        elif isinstance(expr, Syntax.MemberCall):
            receiver = self.processExpr(expr.receiver)
            args = self.processArgs(expr.args)
            call = MethodCall()
            call.receiver = receiver
            call.name = expr.name
            call.args = args
            return self.addInstruction(call)
        elif isinstance(expr, Syntax.FunctionCall):
            args = self.processArgs(expr.args)
            if isinstance(expr.id, Syntax.VarRef):
                call = NamedFunctionCall()
                call.name = expr.id.name
                call.args = args
                return self.addInstruction(call)
            elif isinstance(expr.id, Syntax.TypeRef):
                call = NamedFunctionCall()
                call.name = expr.id.name
                call.args = args
                return self.addInstruction(call)
            else:
                id = self.processExpr(expr.id)
                call = DynamicFunctionCall()
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
                value_ref= ValueRef()
                value_ref.name = var
                value_ref.fields = fields
                ref_id = self.addInstruction(value_ref)
                converter = Converter()
                converter.arg = ref_id
                return self.addInstruction(converter)
            else:
                receiver = self.processExpr(expr.receiver)
                access = MemberAccess()
                access.receiver = receiver
                access.name = expr.name
                return self.addInstruction(access)
        elif isinstance(expr, Syntax.VarRef):
            ref = VarRef()
            ref.name = expr.name
            ref_id = self.addInstruction(ref)
            converter = Converter()
            converter.arg = ref_id
            return self.addInstruction(converter)
        elif isinstance(expr, Syntax.If):
            if_instr = If()
            if_instr.cond = self.processExpr(expr.cond)
            if_instr.true_branch = self.processExpr(expr.true_branch, rootBlock=False, packBlock=False)
            if expr.false_branch:
                if_instr.false_branch = self.processExpr(expr.false_branch, rootBlock=False, packBlock=False)
            return self.addInstruction(if_instr)
        elif isinstance(expr, Syntax.Loop):
            init = self.processExpr(expr.init)
            body = self.processExpr(expr.body, rootBlock=False)
            loop = Loop()
            loop.var = expr.var
            loop.init = init
            loop.body = body
            return self.addInstruction(loop)
        elif isinstance(expr, Syntax.Break):
            arg = self.processExpr(expr.arg)
            br = Break()
            br.arg = arg
            return self.addInstruction(br)
        elif isinstance(expr, Syntax.Continue):
            arg = self.processExpr(expr.arg)
            cont = Continue()
            cont.arg = arg
            return self.addInstruction(cont)
        elif isinstance(expr, Syntax.Return):
            arg = self.processExpr(expr.arg)
            ret = Return()
            ret.arg = arg
            return self.addInstruction(ret)
        elif isinstance(expr, Syntax.BoolLiteral):
            b = BoolLiteral()
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
                processor = Processor()
                block = processor.createBlock()
                for (index, arg) in enumerate(fn.args):
                    arg_name = "arg_%s" % index
                    arg_ref = VarRef()
                    arg_ref.name = arg_name
                    arg_ref_id = block.addInstruction(arg_ref)
                    arg_bind = Bind()
                    arg_bind.name = arg.name
                    arg.name = arg_name
                    arg_bind.rhs = arg_ref_id
                    block.addInstruction(arg_bind)
                processor.processExpr(fn.body, rootBlock=True, packBlock=False)
                body = Body()
                body.blocks = processor.blocks
                fn.body = body
                #fn.body.dump()
            if isinstance(item, Syntax.Class):
                for m in item.methods:
                    #print("Processing method %s" % m.name)
                    processor = Processor()
                    block = processor.createBlock()
                    for (index, arg) in enumerate(m.args):
                        arg_name = "arg_%s" % index
                        arg_ref = VarRef()
                        arg_ref.name = arg_name
                        arg_ref_id = block.addInstruction(arg_ref)
                        arg_bind = Bind()
                        arg_bind.name = arg.name
                        arg.name = arg_name
                        arg_bind.rhs = arg_ref_id
                        block.addInstruction(arg_bind)
                    processor.processExpr(m.body, rootBlock=True, packBlock=False)
                    body = Body()
                    body.blocks = processor.blocks
                    m.body = body
                

                