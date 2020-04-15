use crate::type_processor::process_stored_type;
use siko_mir::expr::Expr;
use siko_mir::expr::ExprId;
use siko_mir::function::Function;
use siko_mir::pattern::Pattern;
use siko_mir::pattern::PatternId;
use siko_mir::program::Program;
use siko_mir::types::DynamicCallTrait;
use siko_mir::types::PartialFunctionCall;
use siko_mir::types::PartialFunctionCallField;
use siko_mir::walker::walk_expr;
use siko_mir::walker::Visitor;

struct ProcessStaticCalls<'a> {
    program: &'a mut Program,
}

impl<'a> Visitor for ProcessStaticCalls<'a> {
    fn get_program(&self) -> &Program {
        return self.program;
    }
    fn visit_expr(&mut self, expr_id: ExprId, expr: &Expr) {
        let location = self.program.exprs.get(&expr_id).location_id;
        match expr {
            Expr::StaticFunctionCall(id, args) => {
                let function: Function = self.program.functions.get(id).clone();
                let mut arg_types = Vec::new();
                function.function_type.get_args(&mut arg_types);
                if function.arg_count > args.len() {
                    let mut fields = Vec::new();
                    let mut traits = Vec::new();
                    for index in 0..function.arg_count {
                        if index < function.arg_count - 1 {
                            let field_type = arg_types[index].clone();
                            let field_type = process_stored_type(field_type, self.program);
                            let field = PartialFunctionCallField { ty: field_type };
                            fields.push(field);
                        }
                        if index >= args.len() {
                            let result_ty = function.function_type.get_result_type(index);
                            let (from, to) = result_ty.get_from_to();
                            let to = process_stored_type(to, self.program);
                            let dyn_trait = if index == function.arg_count - 1 {
                                DynamicCallTrait::RealCall { from: from, to: to }
                            } else {
                                DynamicCallTrait::ArgSave {
                                    from: from,
                                    to: to,
                                    field_index: index,
                                }
                            };
                            traits.push(dyn_trait);
                        }
                    }
                    let partial_function_call_id = self.program.partial_function_calls.get_id();
                    let closure_type = self
                        .program
                        .add_closure_type(&function.function_type.get_result_type(args.len()));
                    let partial_function_call = PartialFunctionCall {
                        id: partial_function_call_id,
                        function: *id,
                        fields: fields,
                        traits: traits,
                        closure_type: closure_type,
                    };
                    self.program
                        .partial_function_calls
                        .add_item(partial_function_call_id, partial_function_call);
                    let partial_function_call_expr =
                        Expr::PartialFunctionCall(partial_function_call_id, args.clone());
                    self.program
                        .update_expr(expr_id, partial_function_call_expr);
                } else if function.arg_count < args.len() {
                    let new_args = args[0..function.arg_count].to_vec();
                    let rest = args[function.arg_count..].to_vec();
                    let new_static_call_expr = Expr::StaticFunctionCall(*id, new_args);
                    let new_function_type =
                        function.function_type.get_result_type(function.arg_count);
                    let new_static_call_expr_id =
                        self.program
                            .add_expr(new_static_call_expr, location, new_function_type);
                    let dynamic_call_expr =
                        Expr::DynamicFunctionCall(new_static_call_expr_id, rest);
                    self.program.update_expr(expr_id, dynamic_call_expr);
                } else {
                    assert_eq!(function.arg_count, args.len());
                }
            }
            _ => {}
        }
    }
    fn visit_pattern(&mut self, _: PatternId, _: &Pattern) {}
}

pub fn process_static_calls_pass(expr_id: &ExprId, program: &mut Program) {
    let mut processor = ProcessStaticCalls { program: program };
    walk_expr(expr_id, &mut processor, false);
}
