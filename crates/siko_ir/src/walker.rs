use crate::expr::Expr;
use crate::expr::ExprId;
use crate::pattern::Pattern;
use crate::pattern::PatternId;
use crate::program::Program;
use std::collections::BTreeSet;

pub trait Visitor {
    fn get_program(&self) -> &Program;
    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr);
    fn visit_pattern(&mut self, pattern_id: PatternId, pattern: &Pattern);
}

pub fn walk_expr(expr_id: &ExprId, visitor: &mut dyn Visitor) {
    let program = visitor.get_program();
    let expr = &program.exprs.get(expr_id).item.clone(); // FIXME, reorganize stuff to remove this clone
    match expr {
        Expr::StaticFunctionCall(_, args) => {
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
        Expr::Tuple(items) => {
            for item in items {
                walk_expr(item, visitor);
            }
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
        Expr::TupleFieldAccess(_, lhs) => {
            walk_expr(lhs, visitor);
        }
        Expr::Formatter(_, items) => {
            for item in items {
                walk_expr(item, visitor);
            }
        }
        Expr::CaseOf(body, cases, _) => {
            walk_expr(body, visitor);
            for case in cases {
                walk_expr(&case.body, visitor);
                walk_pattern(&case.pattern_id, visitor);
            }
        }
        Expr::RecordInitialization(_, items) => {
            for item in items {
                walk_expr(&item.expr_id, visitor);
            }
        }
        Expr::RecordUpdate(record_expr_id, updates) => {
            walk_expr(record_expr_id, visitor);
            let mut visited = BTreeSet::new();
            for update in updates {
                for item in &update.items {
                    if visited.insert(item.expr_id) {
                        walk_expr(&item.expr_id, visitor);
                    }
                }
            }
        }
        Expr::ClassFunctionCall(_, args) => {
            for arg in args {
                walk_expr(arg, visitor);
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
        Pattern::Tuple(items) => {
            for item in items {
                walk_pattern(item, visitor);
            }
        }
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
        Pattern::Typed(id, _) => {
            walk_pattern(id, visitor);
        }
    }
    visitor.visit_pattern(*pattern_id, pattern);
}
