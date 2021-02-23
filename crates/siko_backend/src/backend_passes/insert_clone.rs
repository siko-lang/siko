use siko_mir::expr::Expr;
use siko_mir::expr::ExprId;
use siko_mir::pattern::Pattern;
use siko_mir::pattern::PatternId;
use siko_mir::program::Program;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum VarRef {
    Arg(usize),
    Pattern(PatternId),
}

#[derive(Clone)]
enum Usage {
    Arg(usize),
    Pattern(PatternId),
    Field(Vec<usize>, Box<Usage>),
}

impl Usage {
    fn get_ref(&self) -> VarRef {
        match self {
            Usage::Arg(index) => VarRef::Arg(*index),
            Usage::Pattern(p) => VarRef::Pattern(*p),
            Usage::Field(_, u) => u.get_ref(),
        }
    }

    fn get_fields(&self) -> Vec<usize> {
        match self {
            Usage::Arg(_) => Vec::new(),
            Usage::Pattern(_) => Vec::new(),
            Usage::Field(fields, _) => fields.clone(),
        }
    }
}

struct Collector {
    invalid_refs: Vec<ExprId>,
    patterns: BTreeMap<PatternId, usize>,
    usages: Vec<(Usage, ExprId)>,
    loop_index: usize,
}

fn invalidates(new: &Usage, old: &Usage) -> bool {
    if new.get_ref() == old.get_ref() {
        let new_fields = new.get_fields();
        let old_fields = old.get_fields();
        let len = std::cmp::min(new_fields.len(), old_fields.len());
        new_fields[0..len] == old_fields[0..len]
    } else {
        false
    }
}

impl Collector {
    fn add_usage(&mut self, usage: Usage, expr_id: &ExprId) {
        let mut new_usages = Vec::new();
        std::mem::swap(&mut self.usages, &mut new_usages);
        for (u, e) in new_usages {
            if invalidates(&u, &usage) {
                self.invalid_refs.push(e);
            } else {
                self.usages.push((u, e));
            }
        }
        self.usages.push((usage, *expr_id));
    }
}

fn walk_body(expr_id: &ExprId, program: &Program, collector: &mut Collector, block_index: usize) {
    let expr = &program.exprs.get(expr_id).item;
    match expr {
        Expr::Clone(rhs) => {
            walk_body(rhs, program, collector, block_index);
        }
        Expr::Deref(rhs) => {
            walk_body(rhs, program, collector, block_index);
        }
        Expr::StaticFunctionCall(_, args) => {
            for arg in args {
                walk_body(arg, program, collector, block_index);
            }
        }
        Expr::PartialFunctionCall(_, args) => {
            for arg in args {
                walk_body(arg, program, collector, block_index);
            }
        }
        Expr::DynamicFunctionCall(func_expr, args) => {
            walk_body(func_expr, program, collector, block_index);
            for arg in args {
                walk_body(arg, program, collector, block_index);
            }
        }
        Expr::If(cond, true_branch, false_branch) => {
            walk_body(cond, program, collector, block_index);
            let before = collector.usages.clone();
            walk_body(true_branch, program, collector, block_index);
            let after1 = collector.usages.clone();
            collector.usages = before;
            walk_body(false_branch, program, collector, block_index);
            collector.usages.extend(after1);
        }
        Expr::List(items) => {
            for item in items {
                walk_body(item, program, collector, block_index);
            }
        }
        Expr::IntegerLiteral(_) => {}
        Expr::FloatLiteral(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::CharLiteral(_) => {}
        Expr::Do(items) => {
            for item in items {
                walk_body(item, program, collector, block_index + 1);
            }
        }
        Expr::Bind(bind_pattern, rhs) => {
            walk_body(rhs, program, collector, block_index);
            walk_pattern(bind_pattern, program, collector, block_index);
        }
        Expr::ArgRef(index) => {
            if collector.loop_index > 0 {
                collector.add_usage(Usage::Arg(*index), expr_id);
            }
            collector.add_usage(Usage::Arg(*index), expr_id);
        }
        Expr::ExprValue(_, pattern) => {
            let p_index = *collector
                .patterns
                .get(pattern)
                .expect("patternid not found in collector");
            if p_index < collector.loop_index {
                collector.add_usage(Usage::Pattern(*pattern), expr_id);
            }
            collector.add_usage(Usage::Pattern(*pattern), expr_id);
        }
        Expr::FieldAccess(field_name, lhs) => {
            let mut fields = vec![*field_name];
            let mut lhs = *lhs;
            loop {
                let lhs_expr = &program.exprs.get(&lhs).item;
                match lhs_expr {
                    Expr::FieldAccess(field_name, s_lhs) => {
                        lhs = *s_lhs;
                        fields.push(field_name.clone());
                    }
                    Expr::ArgRef(index) => {
                        let mut fields = fields.clone();
                        fields.reverse();
                        let usage = Usage::Field(fields, Box::new(Usage::Arg(*index)));
                        if collector.loop_index > 0 {
                            collector.add_usage(usage.clone(), expr_id);
                        }
                        collector.add_usage(usage.clone(), expr_id);
                        break;
                    }
                    Expr::ExprValue(_, pattern) => {
                        let mut fields = fields.clone();
                        fields.reverse();
                        let usage = Usage::Field(fields, Box::new(Usage::Pattern(*pattern)));
                        let p_index = *collector
                            .patterns
                            .get(pattern)
                            .expect("patternid not found in collector");
                        if p_index < collector.loop_index {
                            collector.add_usage(usage.clone(), expr_id);
                        }
                        collector.add_usage(usage.clone(), expr_id);
                        break;
                    }
                    _ => {
                        walk_body(&lhs, program, collector, block_index);
                        break;
                    }
                }
            }
        }
        Expr::Formatter(_, items) => {
            for item in items {
                walk_body(item, program, collector, block_index);
            }
        }
        Expr::CaseOf(body, cases) => {
            walk_body(body, program, collector, block_index);
            let mut saved = collector.usages.clone();
            let mut afters = Vec::new();
            for case in cases {
                collector.usages = saved.clone();
                walk_pattern(&case.pattern_id, program, collector, block_index);
                saved = collector.usages.clone();
                walk_body(&case.body, program, collector, block_index);
                afters.push(collector.usages.clone());
            }
            for after in afters {
                collector.usages.extend(after);
            }
        }
        Expr::RecordInitialization(_, items) => {
            for item in items {
                walk_body(&item.0, program, collector, block_index);
            }
        }
        Expr::RecordUpdate(record_expr_id, updates) => {
            walk_body(record_expr_id, program, collector, block_index);
            for item in updates {
                walk_body(&item.0, program, collector, block_index);
            }
        }
        Expr::Return(inner) => {
            walk_body(inner, program, collector, block_index);
        }
        Expr::Loop(pattern, initializer, items, _) => {
            walk_body(initializer, program, collector, block_index);
            walk_pattern(pattern, program, collector, block_index);
            let prev_loop = collector.loop_index;
            collector.loop_index = block_index + 1;
            for item in items {
                walk_body(item, program, collector, block_index + 1);
            }
            collector.loop_index = prev_loop;
        }
        Expr::Continue(inner) => {
            walk_body(inner, program, collector, block_index);
        }
        Expr::Break(inner) => {
            walk_body(inner, program, collector, block_index);
        }
    }
}

fn walk_pattern(
    pattern_id: &PatternId,
    program: &Program,
    collector: &mut Collector,
    block_index: usize,
) {
    let pattern = &program.patterns.get(pattern_id).item;
    match pattern {
        Pattern::Binding(_) => {
            collector.patterns.insert(*pattern_id, block_index);
        }
        Pattern::Record(_, items) => {
            for item in items {
                walk_pattern(item, program, collector, block_index);
            }
        }
        Pattern::Variant(_, _, items) => {
            for item in items {
                walk_pattern(item, program, collector, block_index);
            }
        }
        Pattern::Guarded(id, expr_id) => {
            walk_pattern(id, program, collector, block_index);
            walk_body(expr_id, program, collector, block_index);
        }
        Pattern::Wildcard => {}
        Pattern::IntegerLiteral(_) => {}
        Pattern::StringLiteral(_) => {}
        Pattern::CharLiteral(_) => {}
        Pattern::CharRange(_, _, _) => {}
    }
}

pub fn insert_clone_pass(expr_id: &ExprId, program: &mut Program) {
    let mut collector = Collector {
        invalid_refs: Vec::new(),
        patterns: BTreeMap::new(),
        usages: Vec::new(),
        loop_index: 0,
    };
    walk_body(expr_id, program, &mut collector, 0);

    collector.invalid_refs.sort();
    collector.invalid_refs.dedup();

    for expr_id in collector.invalid_refs.iter() {
        let location = program.exprs.get(&expr_id).location_id;
        let new_ref = program.exprs.get(&expr_id).item.clone();
        let ty = program.get_expr_type(&expr_id).clone();
        let new_ref_id = program.add_expr(new_ref, location, ty);
        let clone = Expr::Clone(new_ref_id);
        program.update_expr(*expr_id, clone);
    }
}
