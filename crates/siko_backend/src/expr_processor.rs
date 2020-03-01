use crate::class_member_processor::process_class_member_call;
use crate::function_queue::CallContext;
use crate::function_queue::FunctionQueue;
use crate::function_queue::FunctionQueueItem;
use crate::pattern_processor::process_pattern;
use crate::type_processor::process_type;
use crate::typedef_store::TypeDefStore;
use siko_ir::expr::Expr as IrExpr;
use siko_ir::expr::ExprId as IrExprId;
use siko_ir::pattern::PatternId as IrPatternId;
use siko_ir::program::Program as IrProgram;
use siko_ir::unifier::Unifier;
use siko_mir::expr::Case as MirCase;
use siko_mir::expr::Expr as MirExpr;
use siko_mir::expr::ExprId as MirExprId;
use siko_mir::pattern::PatternId as MirPatternId;
use siko_mir::program::Program as MirProgram;
use std::collections::BTreeMap;

pub fn process_expr(
    ir_expr_id: &IrExprId,
    ir_program: &IrProgram,
    mir_program: &mut MirProgram,
    unifier: &Unifier,
    function_queue: &mut FunctionQueue,
    typedef_store: &mut TypeDefStore,
    expr_id_map: &mut BTreeMap<IrExprId, MirExprId>,
    pattern_id_map: &mut BTreeMap<IrPatternId, MirPatternId>,
) -> MirExprId {
    let item_info = &ir_program.exprs.get(ir_expr_id);
    let expr = &item_info.item;
    //println!("{} Expr {}", ir_expr_id, expr);
    let mut ir_expr_ty = ir_program.get_expr_type(&ir_expr_id).clone();
    ir_expr_ty.apply(unifier);
    let mir_expr_ty = process_type(&ir_expr_ty, typedef_store, ir_program, mir_program);
    let mir_expr = match expr {
        IrExpr::ArgRef(arg_ref) => {
            assert!(!arg_ref.captured);
            MirExpr::ArgRef(arg_ref.index)
        }
        IrExpr::Bind(pattern_id, rhs) => {
            let mir_pattern_id = process_pattern(
                pattern_id,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            let mir_rhs = process_expr(
                rhs,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            MirExpr::Bind(mir_pattern_id, mir_rhs)
        }
        IrExpr::CaseOf(body, cases, _) => {
            let mir_body = process_expr(
                body,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            let cases: Vec<_> = cases
                .iter()
                .map(|case| {
                    let mir_case_pattern = process_pattern(
                        &case.pattern_id,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    );
                    let mir_case_body = process_expr(
                        &case.body,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    );
                    MirCase {
                        body: mir_case_body,
                        pattern_id: mir_case_pattern,
                    }
                })
                .collect();
            MirExpr::CaseOf(mir_body, cases)
        }
        IrExpr::ClassFunctionCall(class_member_id, args) => {
            let mir_args: Vec<_> = args
                .iter()
                .map(|arg| {
                    process_expr(
                        arg,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            let mut arg_types: Vec<_> = args
                .iter()
                .map(|arg| ir_program.get_expr_type(arg).clone())
                .collect();
            for arg_type in &mut arg_types {
                arg_type.apply(unifier);
            }
            let mir_function_id = process_class_member_call(
                &arg_types,
                ir_program,
                mir_program,
                class_member_id,
                ir_expr_ty,
                function_queue,
            );
            MirExpr::StaticFunctionCall(mir_function_id, mir_args)
        }
        IrExpr::CharLiteral(value) => MirExpr::CharLiteral(*value),
        IrExpr::Do(items) => {
            let mir_items: Vec<_> = items
                .iter()
                .map(|item| {
                    process_expr(
                        item,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            MirExpr::Do(mir_items)
        }
        IrExpr::DynamicFunctionCall(func_expr_id, args) => {
            let mir_func_expr_id = process_expr(
                func_expr_id,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            let mir_args: Vec<_> = args
                .iter()
                .map(|arg| {
                    process_expr(
                        arg,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            MirExpr::DynamicFunctionCall(mir_func_expr_id, mir_args)
        }
        IrExpr::ExprValue(expr_id, pattern_id) => {
            let mir_expr_id = *expr_id_map.get(expr_id).expect("mir_expr_id not found");
            let mir_pattern_id = *pattern_id_map
                .get(pattern_id)
                .expect("mir_pattern_id not found");
            MirExpr::ExprValue(mir_expr_id, mir_pattern_id)
        }
        IrExpr::FieldAccess(infos, receiver_expr_id) => {
            assert_eq!(infos.len(), 1);
            let mir_receiver_expr_id = process_expr(
                receiver_expr_id,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            MirExpr::FieldAccess(infos[0].index, mir_receiver_expr_id)
        }
        IrExpr::FloatLiteral(v) => MirExpr::FloatLiteral(*v),
        IrExpr::Formatter(fmt, args) => {
            let mir_args: Vec<_> = args
                .iter()
                .map(|arg| {
                    process_expr(
                        arg,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            MirExpr::Formatter(fmt.clone(), mir_args)
        }
        IrExpr::If(cond, true_branch, false_branch) => {
            let mir_cond = process_expr(
                cond,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            let mir_true_branch = process_expr(
                true_branch,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            let mir_false_branch = process_expr(
                false_branch,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            MirExpr::If(mir_cond, mir_true_branch, mir_false_branch)
        }
        IrExpr::IntegerLiteral(value) => MirExpr::IntegerLiteral(*value),
        IrExpr::List(items) => {
            let mir_items: Vec<_> = items
                .iter()
                .map(|item| {
                    process_expr(
                        item,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            MirExpr::List(mir_items)
        }
        IrExpr::RecordInitialization(_, fields) => {
            let mir_fields = fields
                .iter()
                .map(|field| {
                    let field_expr = process_expr(
                        &field.expr_id,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    );
                    (field_expr, field.index)
                })
                .collect();
            let mir_id = mir_expr_ty.get_typedef_id();
            MirExpr::RecordInitialization(mir_id, mir_fields)
        }
        IrExpr::RecordUpdate(receiver_expr_id, updates) => {
            let mir_receiver_expr_id = process_expr(
                receiver_expr_id,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            assert_eq!(updates.len(), 1);
            let mir_updates = updates[0]
                .items
                .iter()
                .map(|item| {
                    let field_expr = process_expr(
                        &item.expr_id,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    );
                    (field_expr, item.index)
                })
                .collect();
            MirExpr::RecordUpdate(mir_receiver_expr_id, mir_updates)
        }
        IrExpr::StaticFunctionCall(func_id, args) => {
            let mut arg_types: Vec<_> = args
                .iter()
                .map(|arg| ir_program.get_expr_type(arg).clone())
                .collect();
            for arg_type in &mut arg_types {
                arg_type.apply(unifier);
            }
            let mir_args: Vec<_> = args
                .iter()
                .map(|arg| {
                    process_expr(
                        arg,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            let context = CallContext::new(arg_types, ir_expr_ty.clone());
            let queue_item = FunctionQueueItem::Normal(*func_id, context);
            let mir_function_id = function_queue.insert(queue_item, mir_program);
            MirExpr::StaticFunctionCall(mir_function_id, mir_args)
        }
        IrExpr::StringLiteral(value) => MirExpr::StringLiteral(value.clone()),
        IrExpr::Tuple(items) => {
            let mir_items: Vec<_> = items
                .iter()
                .map(|item| {
                    process_expr(
                        item,
                        ir_program,
                        mir_program,
                        unifier,
                        function_queue,
                        typedef_store,
                        expr_id_map,
                        pattern_id_map,
                    )
                })
                .collect();
            let fields: Vec<_> = mir_items
                .into_iter()
                .enumerate()
                .map(|(index, item)| (item, index))
                .collect();
            let id = mir_expr_ty.get_typedef_id();
            MirExpr::RecordInitialization(id, fields)
        }
        IrExpr::TupleFieldAccess(index, receiver_expr_id) => {
            let mir_receiver_expr_id = process_expr(
                receiver_expr_id,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            MirExpr::FieldAccess(*index, mir_receiver_expr_id)
        }
        IrExpr::Return(inner_id) => {
            let mir_inner_id = process_expr(
                inner_id,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            MirExpr::Return(mir_inner_id)
        }
        IrExpr::Loop(..) => unimplemented!(),
        IrExpr::Break(..) => unimplemented!(),
        IrExpr::Continue(..) => unimplemented!(),
    };
    let mir_expr_id = mir_program.add_expr(mir_expr, item_info.location_id, mir_expr_ty);
    expr_id_map.insert(*ir_expr_id, mir_expr_id);
    mir_expr_id
}
