use siko_ir::class::ClassMemberId;
use siko_ir::expr::Expr;
use siko_ir::expr::ExprId;
use siko_ir::pattern::Pattern;
use siko_ir::pattern::PatternId;
use siko_ir::program::Program;
use siko_ir::walker::Visitor;
use siko_location_info::item::ItemInfo;

pub struct FormatRewriter<'a> {
    program: &'a mut Program,
    show_id: ClassMemberId,
}

impl<'a> FormatRewriter<'a> {
    pub fn new(program: &'a mut Program) -> FormatRewriter<'a> {
        let show_id = program.get_show_member_id();
        FormatRewriter {
            program: program,
            show_id: show_id,
        }
    }
}

impl<'a> Visitor for FormatRewriter<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        match expr {
            Expr::Formatter(fmt, args) => {
                let mut new_args = Vec::new();
                for arg in args {
                    let show_call_expr = Expr::ClassFunctionCall(self.show_id, vec![*arg]);
                    let show_call_expr_id = self.program.exprs.get_id();
                    let location = self.program.exprs.get(arg).location_id;
                    let info = ItemInfo::new(show_call_expr, location);
                    self.program.exprs.add_item(show_call_expr_id, info);
                    let string_ty = self.program.get_string_type();
                    self.program.expr_types.insert(show_call_expr_id, string_ty);
                    new_args.push(show_call_expr_id);
                }
                let expr = &mut self.program.exprs.get_mut(&expr_id).item;
                *expr = Expr::Formatter(fmt.clone(), new_args);
            }

            _ => {}
        }
    }

    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}
