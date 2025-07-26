#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyntaxBlock {
    id: String,
    childBlocks: Vec<SyntaxBlock>,
}

impl SyntaxBlock {
    pub fn new(id: String) -> SyntaxBlock {
        SyntaxBlock {
            id: id,
            childBlocks: Vec::new(),
        }
    }

    pub fn addBlock(&mut self, block: SyntaxBlock) {
        if self.childBlocks.is_empty() {
            self.childBlocks.push(block);
        } else {
            self.childBlocks.last_mut().unwrap().addBlock(block);
        }
    }

    pub fn getCurrentBlockId(&self) -> String {
        if self.childBlocks.is_empty() {
            return format!("{}", self.id);
        } else {
            return format!("{}.{}", self.id, self.childBlocks.last().unwrap().getCurrentBlockId());
        }
    }

    pub fn endBlock(&mut self) {
        assert!(!self.childBlocks.is_empty());
        if self.childBlocks.last().unwrap().childBlocks.is_empty() {
            self.childBlocks.pop();
        } else {
            self.childBlocks.last_mut().unwrap().endBlock();
        }
    }
}
