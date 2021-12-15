use crate::expr::Expr;
use crate::expr::ExprId;
use crate::pattern::Pattern;
use crate::pattern::PatternId;
use crate::program::Program;

pub trait Visitor {
    fn get_program(&self) -> &Program;
    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr);
    fn visit_pattern(&mut self, pattern_id: PatternId, pattern: &Pattern);
}

pub fn walk_expr(expr_id: &ExprId, visitor: &mut dyn Visitor, for_twice: bool) {
    let program = visitor.get_program();
    let expr = &program.exprs.get(expr_id).item.clone(); // FIXME, reorganize stuff to remove this clone
    match expr {
        Expr::Clone(rhs) => {
            walk_expr(rhs, visitor, for_twice);
        }
        Expr::Deref(rhs) => {
            walk_expr(rhs, visitor, for_twice);
        }
        Expr::StaticFunctionCall(_, args) => {
            for arg in args {
                walk_expr(arg, visitor, for_twice);
            }
        }
        Expr::PartialFunctionCall(_, args) => {
            for arg in args {
                walk_expr(arg, visitor, for_twice);
            }
        }
        Expr::DynamicFunctionCall(func_expr, args) => {
            walk_expr(func_expr, visitor, for_twice);
            for arg in args {
                walk_expr(arg, visitor, for_twice);
            }
        }
        Expr::If(cond, true_branch, false_branch) => {
            walk_expr(cond, visitor, for_twice);
            walk_expr(true_branch, visitor, for_twice);
            walk_expr(false_branch, visitor, for_twice);
        }
        Expr::List(items) => {
            for item in items {
                walk_expr(item, visitor, for_twice);
            }
        }
        Expr::IntegerLiteral(_) => {}
        Expr::FloatLiteral(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::CharLiteral(_) => {}
        Expr::Do(items) => {
            for item in items {
                walk_expr(item, visitor, for_twice);
            }
        }
        Expr::Bind(bind_pattern, rhs) => {
            walk_expr(rhs, visitor, for_twice);
            walk_pattern(bind_pattern, visitor, for_twice);
        }
        Expr::ArgRef(_) => {}
        Expr::ExprValue(_, _) => {}
        Expr::FieldAccess(_, lhs) => {
            walk_expr(lhs, visitor, for_twice);
        }
        Expr::Formatter(_, items) => {
            for item in items {
                walk_expr(item, visitor, for_twice);
            }
        }
        Expr::CaseOf(body, cases) => {
            walk_expr(body, visitor, for_twice);
            for case in cases {
                walk_expr(&case.body, visitor, for_twice);
                walk_pattern(&case.pattern_id, visitor, for_twice);
            }
        }
        Expr::RecordInitialization(_, items) => {
            for item in items {
                walk_expr(&item.0, visitor, for_twice);
            }
        }
        Expr::RecordUpdate(record_expr_id, updates) => {
            walk_expr(record_expr_id, visitor, for_twice);
            for item in updates {
                walk_expr(&item.0, visitor, for_twice);
            }
        }
        Expr::Return(inner) => {
            walk_expr(inner, visitor, for_twice);
        }
        Expr::Loop(pattern, initializer, items, _) => {
            walk_expr(initializer, visitor, for_twice);
            walk_pattern(pattern, visitor, for_twice);
            let count = if for_twice { 2 } else { 1 };
            for _ in 0..count {
                for item in items {
                    walk_expr(item, visitor, for_twice);
                }
            }
        }
        Expr::Continue(inner) => {
            walk_expr(inner, visitor, for_twice);
        }
        Expr::Break(inner) => {
            walk_expr(inner, visitor, for_twice);
        }
    }
    visitor.visit_expr(*expr_id, expr);
}

fn walk_pattern(pattern_id: &PatternId, visitor: &mut dyn Visitor, for_twice: bool) {
    let program = visitor.get_program();
    let pattern = &program.patterns.get(pattern_id).item.clone(); // FIXME, reorganize stuff to remove this clone
    match pattern {
        Pattern::Binding(_) => {}
        Pattern::Record(_, items) => {
            for item in items {
                walk_pattern(item, visitor, for_twice);
            }
        }
        Pattern::Variant(_, _, items) => {
            for item in items {
                walk_pattern(item, visitor, for_twice);
            }
        }
        Pattern::Guarded(id, expr_id) => {
            walk_pattern(id, visitor, for_twice);
            walk_expr(expr_id, visitor, for_twice);
        }
        Pattern::Wildcard => {}
        Pattern::IntegerLiteral(_) => {}
        Pattern::StringLiteral(_) => {}
        Pattern::CharLiteral(_) => {}
        Pattern::CharRange(_, _, _) => {}
    }
    visitor.visit_pattern(*pattern_id, pattern);
}
