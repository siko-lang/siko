import IR
import Util

class AddConverter(object):
    def __init__(self):
        self.fn = None
        self.wrapped_instructions = {}

    def wrap(self, i, block):
        converter = IR.Converter()
        converter.arg = i.id
        converter_id = block.addInstruction(converter) 
        self.wrapped_instructions[i.id] = converter_id

    def map(self, i):
        if i in self.wrapped_instructions:
            return self.wrapped_instructions[i]
        else:
            i

    def mapList(self, ids):
        newIds = []
        for i in ids:
            newIds.append(self.map(i))
        return newIds

    def wrapBlock(self, block):
        instructions = block.instructions.copy()
        for i in instructions:
            if isinstance(i, IR.VarRef):
                self.wrap(i, block)
            elif isinstance(i, IR.NamedFunctionCall):
                pass
            elif isinstance(i, IR.Bind):
                pass
            elif isinstance(i, IR.ValueRef):
                self.wrap(i, block)
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.Converter):
                pass
            else:
                Util.error("AddConverter not handling %s %s" % (type(i), i))

    def fixRefs(self, block):
        for i in block.instructions:
            if isinstance(i, IR.VarRef):
                pass
            elif isinstance(i, IR.NamedFunctionCall):
                i.args = self.mapList(i.args)
            elif isinstance(i, IR.Bind):
                pass
            elif isinstance(i, IR.ValueRef):
                self.wrap(i, block)
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.Converter):
                pass
            else:
                Util.error("AddConverter not handling %s %s" % (type(i), i))

    def process(self, fn):
        self.fn = fn
        for block in self.fn.body.blocks:
            self.wrapBlock(block)

        for block in self.fn.body.blocks:
            self.fixRefs(block)

    def dump(self):
        self.fn.body.dump()

def process(program):
    for f in program.functions.values():
        addConverter = AddConverter()
        addConverter.process(f)
        addConverter.dump()
