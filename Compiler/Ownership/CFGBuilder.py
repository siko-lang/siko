import Compiler.IR.Instruction as Instruction
import Compiler.Util as Util
import Compiler.Ownership.CFG as CFG
import Compiler.Ownership.Path as Path

class CFGBuilder(object):
    def __init__(self):
        self.cfg = None
        self.fn = None
        self.end_key = CFG.EndKey()
        self.loop_starts = []
        self.loop_ends = []

    def processGenericInstruction(self, i, last):
        instr_key = CFG.InstructionKey()
        instr_key.id = i.id
        instr_node = CFG.Node()
        instr_node.kind = str(i)
        self.cfg.addNode(instr_key, instr_node)
        if last:
            edge = CFG.Edge()
            edge.from_node = last
            edge.to_node = instr_key
            self.cfg.addEdge(edge)
        return instr_key

    def processBlock(self, block, last = None):
        for i in block.instructions:
            if isinstance(i, Instruction.BlockRef):
                b = self.fn.body.getBlock(i)
                last = self.processBlock(b, last)
            elif isinstance(i, Instruction.NamedFunctionCall):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, Instruction.Tuple):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, Instruction.MethodCall):
                Util.error("method call in CFG!")
            elif isinstance(i, Instruction.Bind):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, Instruction.DropVar):
                instr_key = CFG.DropKey()
                instr_key.id = i.id
                instr_node = CFG.Node()
                instr_node.kind = str(i)
                instr_node.usage = Path.WholePath(isDrop=True)
                instr_node.usage.var = i.name
                self.cfg.addNode(instr_key, instr_node)
                if last:
                    edge = CFG.Edge()
                    edge.from_node = last
                    edge.to_node = instr_key
                    self.cfg.addEdge(edge)
                last = instr_key
            elif isinstance(i, Instruction.MemberAccess):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, Instruction.ValueRef):
                instr_key = CFG.InstructionKey()
                instr_key.id = i.id
                instr_node = CFG.Node()
                instr_node.kind = str(i)
                if len(i.fields) == 0:
                    instr_node.usage = Path.WholePath()
                else:
                    instr_node.usage = Path.PartialPath()
                instr_node.usage.var = i.name
                instr_node.usage.fields = i.fields
                self.cfg.addNode(instr_key, instr_node)
                if last:
                    edge = CFG.Edge()
                    edge.from_node = last
                    edge.to_node = instr_key
                    self.cfg.addEdge(edge)
                last = instr_key
            elif isinstance(i, Instruction.BoolLiteral):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, Instruction.Return):
                self.processGenericInstruction(i, last)
                last = None
            elif isinstance(i, Instruction.Break):
                instr_key = CFG.InstructionKey()
                instr_key.id = i.id
                instr_node = CFG.Node()
                instr_node.kind = str(i)
                self.cfg.addNode(instr_key, instr_node)
                edge = CFG.Edge()
                edge.from_node = instr_key
                edge.to_node = self.loop_ends[-1]
                self.cfg.addEdge(edge)
                if last:
                    edge = CFG.Edge()
                    edge.from_node = last
                    edge.to_node = instr_key
                    self.cfg.addEdge(edge)
                last = None
            elif isinstance(i, Instruction.Continue):
                instr_key = CFG.InstructionKey()
                instr_key.id = i.id
                instr_node = CFG.Node()
                instr_node.kind = str(i)
                self.cfg.addNode(instr_key, instr_node)
                edge = CFG.Edge()
                edge.from_node = instr_key
                edge.to_node = self.loop_starts[-1]
                self.cfg.addEdge(edge)
                if last:
                    edge = CFG.Edge()
                    edge.from_node = last
                    edge.to_node = instr_key
                    self.cfg.addEdge(edge)
                last = None
            elif isinstance(i, Instruction.Loop):
                loop_start_key = CFG.LoopStart()
                loop_start_key.id = i.id
                loop_start_node = CFG.Node()
                loop_start_node.kind = "loop_start"
                self.loop_starts.append(loop_start_key)
                self.cfg.addNode(loop_start_key, loop_start_node)
                if last:
                    edge = CFG.Edge()
                    edge.from_node = last
                    edge.to_node = loop_start_key
                    self.cfg.addEdge(edge)
                loop_var_key = CFG.LoopStart()
                loop_var_key.id = i.id
                loop_var_node = CFG.Node()
                loop_var_node.kind = "loop_var %s" % i.var
                self.cfg.addNode(loop_var_key, loop_var_node)
                edge = CFG.Edge()
                edge.from_node = loop_start_key
                edge.to_node = loop_var_key
                self.cfg.addEdge(edge)
                loop_end_key = CFG.LoopEnd()
                loop_end_key.id = i.id
                loop_end_node = CFG.Node()
                loop_end_node.kind = "loop_end"
                self.cfg.addNode(loop_end_key, loop_end_node)
                self.loop_ends.append(loop_end_key)
                loop_body = self.fn.body.getBlock(i.body)
                loop_last = self.processBlock(loop_body, loop_var_key)
                if loop_last:
                    edge = CFG.Edge()
                    edge.from_node = loop_last
                    edge.to_node = loop_start_key
                    self.cfg.addEdge(edge)
                self.loop_starts.pop()
                self.loop_ends.pop()
                last = loop_end_key
            elif isinstance(i, Instruction.If):
                if_key = CFG.IfKey()
                if_key.id = i.id
                if_end = CFG.Node()
                if_end.kind = "if_end"
                self.cfg.addNode(if_key, if_end)
                true_block = self.fn.body.getBlock(i.true_branch)
                true_last = self.processBlock(true_block, last)
                if true_last:
                    edge = CFG.Edge()
                    edge.from_node = true_last
                    edge.to_node = if_key
                    self.cfg.addEdge(edge)
                false_block = self.fn.body.getBlock(i.false_branch)
                false_last = self.processBlock(false_block, last)
                if false_last:
                    edge = CFG.Edge()
                    edge.from_node = false_last
                    edge.to_node = if_key
                    self.cfg.addEdge(edge)
                last = if_key
            else:
                Util.error("Unhandled in cfg %s: %s" % (type(i), i))
        return last

    def build(self, fn):
        self.fn = fn
        self.cfg = CFG.CFG(fn.name)
        self.processBlock(self.fn.body.getFirst())
        self.cfg.updateEdges()
        return self.cfg

