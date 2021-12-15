use crate::environment::NamedRef;
use siko_ir::expr::Expr;
use siko_ir::expr::FunctionArgumentRef;
use siko_ir::function::FunctionId;
use siko_util::Counter;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct LambdaHelperInner {
    captures: Vec<Expr>,
    level: usize,
    host_function_name: String,
    counter: Rc<RefCell<Counter>>,
    function_id: FunctionId,
    host_function_id: FunctionId,
    parent: Option<LambdaHelper>,
}

impl LambdaHelperInner {
    fn process_named_ref(&mut self, r: NamedRef, level: usize) -> Expr {
        let r = if let Some(parent) = &self.parent {
            parent.process_named_ref(r, level)
        } else {
            match r {
                NamedRef::ExprValue(expr_ref, pattern_id) => Expr::ExprValue(expr_ref, pattern_id),
                NamedRef::FunctionArg(arg_ref) => Expr::ArgRef(arg_ref),
            }
        };
        if level < self.level {
            //println!("capturing {:?}", r);
            let arg_index = self.captures.len();
            let lambda_arg_ref = FunctionArgumentRef::new(true, self.function_id, arg_index);
            let updated_ref = match &r {
                Expr::ExprValue(_, _) => Expr::ArgRef(lambda_arg_ref),
                Expr::ArgRef(_) => Expr::ArgRef(lambda_arg_ref),
                _ => panic!("Unexpected name ref {:?}", r),
            };
            //println!("Captured variable {:?}", updated_ref);
            self.captures.push(r);
            updated_ref
        } else {
            r
        }
    }

    fn captures(&self) -> Vec<Expr> {
        self.captures.clone()
    }

    fn host_function_name(&self) -> String {
        self.host_function_name.clone()
    }

    fn host_function(&self) -> FunctionId {
        self.host_function_id
    }

    fn get_lambda_index(&self) -> usize {
        let index = self.counter.borrow_mut().next();
        index
    }

    fn clone_counter(&self) -> Rc<RefCell<Counter>> {
        self.counter.clone()
    }
}

#[derive(Debug, Clone)]
pub struct LambdaHelper {
    inner: Rc<RefCell<LambdaHelperInner>>,
}

impl LambdaHelper {
    pub fn new(
        level: usize,
        host_function_name: String,
        counter: Rc<RefCell<Counter>>,
        function_id: FunctionId,
        host_function_id: FunctionId,
        parent: Option<LambdaHelper>,
    ) -> LambdaHelper {
        let inner = LambdaHelperInner {
            captures: Vec::new(),
            level: level,
            host_function_name: host_function_name,
            counter: counter,
            function_id: function_id,
            host_function_id: host_function_id,
            parent: parent,
        };
        LambdaHelper {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn process_named_ref(&self, r: NamedRef, level: usize) -> Expr {
        self.inner.borrow_mut().process_named_ref(r, level)
    }

    pub fn captures(&self) -> Vec<Expr> {
        self.inner.borrow().captures()
    }

    pub fn host_function_name(&self) -> String {
        self.inner.borrow().host_function_name()
    }

    pub fn host_function(&self) -> FunctionId {
        self.inner.borrow().host_function()
    }

    pub fn get_lambda_index(&self) -> usize {
        self.inner.borrow_mut().get_lambda_index()
    }

    pub fn new_counter() -> Rc<RefCell<Counter>> {
        Rc::new(RefCell::new(Counter::new()))
    }

    pub fn clone_counter(&self) -> Rc<RefCell<Counter>> {
        self.inner.borrow().clone_counter()
    }
}
