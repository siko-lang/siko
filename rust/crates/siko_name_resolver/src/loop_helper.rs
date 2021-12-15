use siko_ir::expr::ExprId;
use std::cell::RefCell;
use std::rc::Rc;

struct Inner {
    pub continues: Vec<ExprId>,
    pub breaks: Vec<ExprId>,
}

impl Inner {
    fn new() -> Inner {
        Inner {
            continues: Vec::new(),
            breaks: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct LoopHelper {
    inner: Rc<RefCell<Inner>>,
}

impl LoopHelper {
    pub fn new() -> LoopHelper {
        LoopHelper {
            inner: Rc::new(RefCell::new(Inner::new())),
        }
    }

    pub fn add_continue(&self, cont_expr: ExprId) {
        let mut i = self.inner.borrow_mut();
        i.continues.push(cont_expr);
    }

    pub fn add_break(&self, break_expr: ExprId) {
        let mut i = self.inner.borrow_mut();
        i.breaks.push(break_expr);
    }

    pub fn get(&self) -> (Vec<ExprId>, Vec<ExprId>) {
        let i = self.inner.borrow();
        (i.continues.clone(), i.breaks.clone())
    }
}
