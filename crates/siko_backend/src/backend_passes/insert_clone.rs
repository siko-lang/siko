use siko_mir::expr::Expr;
use siko_mir::expr::ExprId;
use siko_mir::pattern::Pattern;
use siko_mir::pattern::PatternId;
use siko_mir::program::Program;
use siko_mir::walker::walk_expr;
use siko_mir::walker::Visitor;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum VarRef {
    Arg(usize),
    Pattern(PatternId),
}

struct VarRefCollector<'a> {
    program: &'a mut Program,
    refs: BTreeMap<VarRef, Vec<ExprId>>,
}

impl<'a> Visitor for VarRefCollector<'a> {
    fn get_program(&self) -> &Program {
        return self.program;
    }
    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        match expr {
            Expr::ArgRef(index) => {
                let refs = self
                    .refs
                    .entry(VarRef::Arg(*index))
                    .or_insert_with(|| Vec::new());
                if !refs.is_empty() {
                    panic!("Arg ref {} used multiple times");
                }
                refs.push(expr_id);
            }
            Expr::ExprValue(_, pattern) => {
                let refs = self
                    .refs
                    .entry(VarRef::Pattern(*pattern))
                    .or_insert_with(|| Vec::new());
                refs.push(expr_id);
            }
            _ => {}
        }
    }
    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}

pub fn insert_clone_pass(expr_id: &ExprId, program: &mut Program) {
    let mut collector = VarRefCollector {
        program: program,
        refs: BTreeMap::new(),
    };
    walk_expr(expr_id, &mut collector, true);
    let refs = collector.refs;
    for (_, exprs) in refs {
        if exprs.len() == 1 {
            continue;
        }
        for (index, expr_id) in exprs.iter().enumerate() {
            if index == exprs.len() -1 {
                break;
            }
            let location = program.exprs.get(&expr_id).location_id;
            let new_ref = program.exprs.get(&expr_id).item.clone();
            let ty = program.get_expr_type(&expr_id).clone();
            let new_ref_id = program.add_expr(new_ref, location, ty);
            let clone = Expr::Clone(new_ref_id);
            program.update_expr(*expr_id, clone);
        }
    }
}
