use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::expr::FunctionArgumentRef;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::walker::Visitor;

pub struct LambdaArgShifter<'a> {
    program: &'a mut Program,
    capture_count: usize,
}

impl<'a> LambdaArgShifter<'a> {
    pub fn new(program: &'a mut Program, capture_count: usize) -> LambdaArgShifter<'a> {
        LambdaArgShifter {
            program: program,
            capture_count: capture_count,
        }
    }
}

impl<'a> Visitor for LambdaArgShifter<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        match expr {
            Expr::ArgRef(arg_ref) => {
                let new_offset = if arg_ref.captured {
                    arg_ref.index
                } else {
                    arg_ref.index + self.capture_count
                };
                let arg_ref = FunctionArgumentRef::new(false, arg_ref.id, new_offset);
                self.program.update_arg_ref(&expr_id, arg_ref);
            }

            _ => {}
        }
    }

    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}
