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

pub fn walk_expr(expr_id: &ExprId, visitor: &mut dyn Visitor) {
    let program = visitor.get_program();
    let expr = &program.exprs.get(expr_id).item.clone(); // FIXME, reorganize stuff to remove this clone
    match expr {
        Expr::Clone(rhs) => {
            walk_expr(rhs, visitor);
        }
        Expr::Deref(rhs) => {
            walk_expr(rhs, visitor);
        }
        Expr::StaticFunctionCall(_, args) => {
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        Expr::PartialFunctionCall(_, args) => {
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        Expr::DynamicFunctionCall(func_expr, args) => {
            walk_expr(func_expr, visitor);
            for arg in args {
                walk_expr(arg, visitor);
            }
        }
        Expr::If(cond, true_branch, false_branch) => {
            walk_expr(cond, visitor);
            walk_expr(true_branch, visitor);
            walk_expr(false_branch, visitor);
        }
        Expr::List(items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::IntegerLiteral(_) => {}
        Expr::FloatLiteral(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::CharLiteral(_) => {}
        Expr::Do(items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::Bind(bind_pattern, rhs) => {
            walk_expr(rhs, visitor);
            walk_pattern(bind_pattern, visitor);
        }
        Expr::ArgRef(_) => {}
        Expr::ExprValue(_, _) => {}
        Expr::FieldAccess(_, lhs) => {
            walk_expr(lhs, visitor);
        }
        Expr::Formatter(_, items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::CaseOf(body, cases) => {
            walk_expr(body, visitor);
            for case in cases {
                walk_expr(&case.body, visitor);
                walk_pattern(&case.pattern_id, visitor);
            }
        }
        Expr::RecordInitialization(_, items) => {
            for item in items {
                walk_expr(&item.0, visitor);
            }
        }
        Expr::RecordUpdate(record_expr_id, updates) => {
            walk_expr(record_expr_id, visitor);
            for item in updates {
                walk_expr(&item.0, visitor);
            }
        }
    }
    visitor.visit_expr(*expr_id, expr);
}

fn walk_pattern(pattern_id: &PatternId, visitor: &mut dyn Visitor) {
    let program = visitor.get_program();
    let pattern = &program.patterns.get(pattern_id).item.clone(); // FIXME, reorganize stuff to remove this clone
    match pattern {
        Pattern::Binding(_) => {}
        Pattern::Record(_, items) => {
            for item in items {
                walk_pattern(item, visitor);
            }
        }
        Pattern::Variant(_, _, items) => {
            for item in items {
                walk_pattern(item, visitor);
            }
        }
        Pattern::Guarded(id, expr_id) => {
            walk_pattern(id, visitor);
            walk_expr(expr_id, visitor);
        }
        Pattern::Wildcard => {}
        Pattern::IntegerLiteral(_) => {}
        Pattern::StringLiteral(_) => {}
        Pattern::CharLiteral(_) => {}
        Pattern::CharRange(_, _, _) => {}
    }
    visitor.visit_pattern(*pattern_id, pattern);
}
