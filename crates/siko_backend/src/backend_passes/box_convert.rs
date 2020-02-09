use siko_mir::expr::Expr;
use siko_mir::expr::ExprId;
use siko_mir::pattern::Pattern;
use siko_mir::pattern::PatternId;
use siko_mir::program::Program;
use siko_mir::types::Type;
use siko_mir::walker::walk_expr;
use siko_mir::walker::Visitor;

struct RetypePattern<'a> {
    program: &'a mut Program,
    patterns: Vec<PatternId>,
}

impl<'a> Visitor for RetypePattern<'a> {
    fn get_program(&self) -> &Program {
        return self.program;
    }
    fn visit_expr(&mut self, _: ExprId, _: &Expr) {}

    fn visit_pattern(&mut self, _: PatternId, pattern: &Pattern) {
        match pattern {
            Pattern::Variant(id, index, items) => {
                let adt = self.program.typedefs.get(id).get_adt();
                let variant = &adt.variants[*index];
                for (item, variant_item) in items.iter().zip(variant.items.iter()) {
                    if variant_item.is_boxed() {
                        self.patterns.push(*item);
                    }
                }
            }
            Pattern::Record(id, items) => {
                let record = self.program.typedefs.get(id).get_record();
                for (item, field) in items.iter().zip(record.fields.iter()) {
                    if field.ty.is_boxed() {
                        self.patterns.push(*item);
                    }
                }
            }
            _ => {}
        }
    }
}

struct VarRefCollector<'a> {
    program: &'a mut Program,
    refs: Vec<ExprId>,
}

impl<'a> Visitor for VarRefCollector<'a> {
    fn get_program(&self) -> &Program {
        return self.program;
    }
    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        match expr {
            Expr::ExprValue(_, pattern) => {
                let ty = self.program.get_pattern_type(pattern);
                if ty.is_boxed() {
                    self.refs.push(expr_id);
                }
            }
            _ => {}
        }
    }

    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}

pub fn box_convert_pass(expr_id: &ExprId, program: &mut Program) {
    let mut retype = RetypePattern {
        program: program,
        patterns: Vec::new(),
    };
    walk_expr(expr_id, &mut retype);
    let patterns = retype.patterns;
    for pattern in patterns {
        let ty = program
            .pattern_types
            .get_mut(&pattern)
            .expect("Pattern type not found");
        *ty = Type::Boxed(Box::new(ty.clone()));
    }
    let mut collector = VarRefCollector {
        program: program,
        refs: Vec::new(),
    };
    walk_expr(expr_id, &mut collector);
    let refs = collector.refs;
    for expr_id in refs {
        let location = program.exprs.get(&expr_id).location_id;
        let new_ref = program.exprs.get(&expr_id).item.clone();
        let ty = program.get_expr_type(&expr_id).clone();
        let new_ref_id = program.add_expr(new_ref, location, ty);
        let deref = Expr::Deref(new_ref_id);
        program.update_expr(expr_id, deref);
    }
}
