use crate::expr_processor::process_expr;
use crate::function_queue::FunctionQueue;
use crate::type_processor::process_type;
use crate::typedef_store::TypeDefStore;
use siko_ir::expr::ExprId as IrExprId;
use siko_ir::pattern::Pattern as IrPattern;
use siko_ir::pattern::PatternId as IrPatternId;
use siko_ir::pattern::RangeKind as IrRangeKind;
use siko_ir::program::Program as IrProgram;
use siko_ir::unifier::Unifier;
use siko_mir::expr::ExprId as MirExprId;
use siko_mir::pattern::Pattern as MirPattern;
use siko_mir::pattern::PatternId as MirPatternId;
use siko_mir::pattern::RangeKind as MirRangeKind;
use siko_mir::program::Program as MirProgram;
use std::collections::BTreeMap;

fn to_mir_kind(kind: &IrRangeKind) -> MirRangeKind {
    match kind {
        IrRangeKind::Exclusive => MirRangeKind::Exclusive,
        IrRangeKind::Inclusive => MirRangeKind::Inclusive,
    }
}

pub fn process_pattern(
    ir_pattern_id: &IrPatternId,
    ir_program: &IrProgram,
    mir_program: &mut MirProgram,
    unifier: &Unifier,
    function_queue: &mut FunctionQueue,
    typedef_store: &mut TypeDefStore,
    expr_id_map: &mut BTreeMap<IrExprId, MirExprId>,
    pattern_id_map: &mut BTreeMap<IrPatternId, MirPatternId>,
) -> MirPatternId {
    let item_info = &ir_program.patterns.get(ir_pattern_id);
    let mut ir_pattern_ty = ir_program.get_pattern_type(ir_pattern_id).clone();
    ir_pattern_ty.apply(unifier);
    let mir_pattern_ty = process_type(&ir_pattern_ty, typedef_store, ir_program, mir_program);
    let pattern = &item_info.item;
    let mir_pattern = match pattern {
        IrPattern::Binding(name) => MirPattern::Binding(name.clone()),
        IrPattern::Guarded(sub, guard_expr) => {
            let mir_sub = process_pattern(
                sub,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            let mir_guard_expr = process_expr(
                guard_expr,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
            MirPattern::Guarded(mir_sub, mir_guard_expr)
        }
        IrPattern::IntegerLiteral(v) => MirPattern::IntegerLiteral(v.clone()),
        IrPattern::CharLiteral(v) => MirPattern::CharLiteral(v.clone()),
        IrPattern::CharRange(start, end, kind) => {
            MirPattern::CharRange(start.clone(), end.clone(), to_mir_kind(kind))
        }
        IrPattern::Record(_, items) => {
            let mir_typedef_id = typedef_store.add_type(ir_pattern_ty, ir_program, mir_program);
            let mir_items: Vec<_> = items
                .iter()
                .map(|item| {
                    process_pattern(
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
            MirPattern::Record(mir_typedef_id, mir_items)
        }
        IrPattern::StringLiteral(v) => MirPattern::StringLiteral(v.clone()),
        IrPattern::Tuple(items) => {
            let mir_typedef_id = typedef_store.add_type(ir_pattern_ty, ir_program, mir_program);
            let mir_items: Vec<_> = items
                .iter()
                .map(|item| {
                    process_pattern(
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
            MirPattern::Record(mir_typedef_id, mir_items)
        }
        IrPattern::Typed(sub, _) => {
            return process_pattern(
                sub,
                ir_program,
                mir_program,
                unifier,
                function_queue,
                typedef_store,
                expr_id_map,
                pattern_id_map,
            );
        }
        IrPattern::Variant(_, index, items) => {
            let mir_typedef_id = typedef_store.add_type(ir_pattern_ty, ir_program, mir_program);
            let mir_items: Vec<_> = items
                .iter()
                .map(|item| {
                    process_pattern(
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
            MirPattern::Variant(mir_typedef_id, *index, mir_items)
        }
        IrPattern::Wildcard => MirPattern::Wildcard,
    };
    let mir_pattern_id =
        mir_program.add_pattern(mir_pattern, item_info.location_id, mir_pattern_ty);
    pattern_id_map.insert(*ir_pattern_id, mir_pattern_id);
    mir_pattern_id
}
