use siko_mir::expr::ExprId;
use siko_mir::pattern::Pattern;
use siko_mir::pattern::PatternId;
use siko_mir::program::Program;
use siko_mir::{data::TypeDef, expr::Expr};

struct Collector {
    updates: Vec<(ExprId, ExprId, Vec<(ExprId, usize)>)>,
}

fn walk_body(expr_id: &ExprId, program: &Program, collector: &mut Collector) {
    let expr = &program.exprs.get(expr_id).item;
    match expr {
        Expr::Clone(rhs) => {
            walk_body(rhs, program, collector);
        }
        Expr::Deref(rhs) => {
            walk_body(rhs, program, collector);
        }
        Expr::StaticFunctionCall(_, args) => {
            for arg in args {
                walk_body(arg, program, collector);
            }
        }
        Expr::PartialFunctionCall(_, args) => {
            for arg in args {
                walk_body(arg, program, collector);
            }
        }
        Expr::DynamicFunctionCall(func_expr, args) => {
            walk_body(func_expr, program, collector);
            for arg in args {
                walk_body(arg, program, collector);
            }
        }
        Expr::If(cond, true_branch, false_branch) => {
            walk_body(cond, program, collector);
            walk_body(true_branch, program, collector);
            walk_body(false_branch, program, collector);
        }
        Expr::List(items) => {
            for item in items {
                walk_body(item, program, collector);
            }
        }
        Expr::IntegerLiteral(_) => {}
        Expr::FloatLiteral(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::CharLiteral(_) => {}
        Expr::Do(items) => {
            for item in items {
                walk_body(item, program, collector);
            }
        }
        Expr::Bind(bind_pattern, rhs) => {
            walk_body(rhs, program, collector);
            walk_pattern(bind_pattern, program, collector);
        }
        Expr::ArgRef(_) => {}
        Expr::ExprValue(_, _) => {}
        Expr::FieldAccess(_, lhs) => {
            walk_body(lhs, program, collector);
        }
        Expr::Formatter(_, items) => {
            for item in items {
                walk_body(item, program, collector);
            }
        }
        Expr::CaseOf(body, cases) => {
            walk_body(body, program, collector);
            for case in cases {
                walk_pattern(&case.pattern_id, program, collector);
                walk_body(&case.body, program, collector);
            }
        }
        Expr::RecordInitialization(_, items) => {
            for item in items {
                walk_body(&item.0, program, collector);
            }
        }
        Expr::RecordUpdate(record_expr_id, updates) => {
            collector
                .updates
                .push((*expr_id, *record_expr_id, updates.clone()));
            walk_body(record_expr_id, program, collector);
            for item in updates {
                walk_body(&item.0, program, collector);
            }
        }
        Expr::Return(inner) => {
            walk_body(inner, program, collector);
        }
        Expr::Loop(pattern, initializer, items, _) => {
            walk_body(initializer, program, collector);
            walk_pattern(pattern, program, collector);
            for _ in 0..2 {
                for item in items {
                    walk_body(item, program, collector);
                }
            }
        }
        Expr::Continue(inner) => {
            walk_body(inner, program, collector);
        }
        Expr::Break(inner) => {
            walk_body(inner, program, collector);
        }
    }
}

fn walk_pattern(pattern_id: &PatternId, program: &Program, collector: &mut Collector) {
    let pattern = &program.patterns.get(pattern_id).item;
    match pattern {
        Pattern::Binding(_) => {}
        Pattern::Record(_, items) => {
            for item in items {
                walk_pattern(item, program, collector);
            }
        }
        Pattern::Variant(_, _, items) => {
            for item in items {
                walk_pattern(item, program, collector);
            }
        }
        Pattern::Guarded(id, expr_id) => {
            walk_pattern(id, program, collector);
            walk_body(expr_id, program, collector);
        }
        Pattern::Wildcard => {}
        Pattern::IntegerLiteral(_) => {}
        Pattern::StringLiteral(_) => {}
        Pattern::CharLiteral(_) => {}
        Pattern::CharRange(_, _, _) => {}
    }
}

pub fn transform_updates_pass(expr_id: &ExprId, program: &mut Program) {
    let mut collector = Collector {
        updates: Vec::new(),
    };
    walk_body(expr_id, program, &mut collector);

    for (expr_id, record_expr_id, updates) in collector.updates.iter() {
        let ty = program.expr_types.get(expr_id).unwrap();
        let record_id = ty.get_typedef_id();
        let record = match program.typedefs.get(&record_id) {
            TypeDef::Record(record) => record.clone(),
            _ => {
                unreachable!();
            }
        };
        let location = program.exprs.get(&expr_id).location_id;
        let mut args = vec![];
        for (index, field) in record.fields.iter().enumerate() {
            let mut found = false;
            for (update_expr, field_index) in updates {
                if *field_index == index {
                    args.push((*update_expr, index));
                    found = true;
                    break;
                }
            }
            if !found {
                let fa = Expr::FieldAccess(index, *record_expr_id);
                let new_field_access = program.add_expr(fa, location, field.ty.clone());
                args.push((new_field_access, index));
            }
        }
        let call = Expr::RecordInitialization(record_id, args);
        program.update_expr(*expr_id, call);
    }
}
