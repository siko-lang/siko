use crate::expr::Expr;
use crate::expr::ExprId;
use crate::function::FunctionId;
use crate::pattern::Pattern;
use crate::pattern::PatternId;
use crate::program::Program;
use crate::walker::walk_expr;
use crate::walker::Visitor;
use siko_util::dependency_processor::DependencyCollector;
use siko_util::dependency_processor::DependencyGroup;
use siko_util::dependency_processor::DependencyProcessor;
use std::collections::BTreeSet;

struct FunctionDependencyCollector<'a> {
    program: &'a Program,
    used_functions: BTreeSet<FunctionId>,
}

impl<'a> FunctionDependencyCollector<'a> {
    fn new(program: &'a Program) -> FunctionDependencyCollector<'a> {
        FunctionDependencyCollector {
            program: program,
            used_functions: BTreeSet::new(),
        }
    }
}

impl<'a> Visitor for FunctionDependencyCollector<'a> {
    fn get_program(&self) -> &Program {
        &self.program
    }

    fn visit_expr(&mut self, _: ExprId, expr: &Expr) {
        match expr {
            Expr::StaticFunctionCall(id, _) => {
                self.used_functions.insert(*id);
            }
            _ => {}
        }
    }

    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {
        // do nothing
    }
}

pub struct FunctionDependencyProcessor<'a> {
    program: &'a Program,
}

impl<'a> FunctionDependencyProcessor<'a> {
    pub fn new(program: &'a Program) -> FunctionDependencyProcessor<'a> {
        FunctionDependencyProcessor { program: program }
    }

    pub fn process_functions(&self) -> Vec<DependencyGroup<FunctionId>> {
        let mut functions = Vec::new();
        for (id, function) in &self.program.functions.items {
            if let Some(_) = function.get_body() {
                functions.push(*id);
            }
        }

        let dep_processor = DependencyProcessor::new(functions);
        let ordered_function_groups = dep_processor.process_items(self);

        ordered_function_groups
    }
}

impl<'a> DependencyCollector<FunctionId> for FunctionDependencyProcessor<'a> {
    fn collect(&self, function_id: FunctionId) -> Vec<FunctionId> {
        let function = self.program.functions.get(&function_id);
        let body = function.get_body().unwrap();
        let mut collector = FunctionDependencyCollector::new(self.program);
        walk_expr(&body, &mut collector);
        let deps: Vec<_> = collector.used_functions.into_iter().collect();
        //println!("{} deps {}", id, format_list(&deps[..]));
        let mut deps: BTreeSet<_> = deps
            .iter()
            .filter(|dep_id| {
                let function = self.program.functions.get(dep_id);
                !function.is_typed()
            })
            .map(|id| *id)
            .collect();
        let func_info = self.program.functions.get(&function_id);
        if let Some(host) = func_info.get_lambda_host() {
            deps.insert(host);
        }
        deps.into_iter().collect()
    }
}
