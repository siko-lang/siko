use crate::expr_processor::process_expr;
use crate::function_queue::FunctionQueue;
use crate::function_queue::FunctionQueueItem;
use crate::type_processor::process_type;
use crate::typedef_store::TypeDefStore;
use crate::util::get_call_unifier;
use crate::util::preprocess_ir;
use siko_ir::data::TypeDef;
use siko_ir::function::FunctionId as IrFunctionId;
use siko_ir::function::FunctionInfo;
use siko_ir::function::NamedFunctionKind;
use siko_ir::program::Program as IrProgram;
use siko_ir::types::Type;
use siko_mir::function::Function as MirFunction;
use siko_mir::function::FunctionId as MirFunctionId;
use siko_mir::function::FunctionInfo as MirFunctionInfo;
use siko_mir::program::Program as MirProgram;
use std::collections::BTreeMap;

pub fn process_function(
    ir_function_id: &IrFunctionId,
    mir_function_id: MirFunctionId,
    ir_program: &mut IrProgram,
    mir_program: &mut MirProgram,
    arg_types: Vec<Type>,
    result_ty: Type,
    function_queue: &mut FunctionQueue,
    typedef_store: &mut TypeDefStore,
) {
    let mut function_type = ir_program
        .get_function_type(ir_function_id)
        .remove_fixed_types();
    let call_unifier = get_call_unifier(&arg_types, &function_type, &result_ty, ir_program);
    function_type.apply(&call_unifier);
    let mir_function_type = process_type(&function_type, typedef_store, ir_program, mir_program);
    let function = ir_program.functions.get(ir_function_id).clone();
    //println!("Processing {}", function.info);
    match &function.info {
        FunctionInfo::NamedFunction(info) => {
            let mir_function_info = if let Some(body) = info.body {
                preprocess_ir(body, ir_program);
                let mut expr_id_map = BTreeMap::new();
                let mut pattern_id_map = BTreeMap::new();
                let mir_expr_id = process_expr(
                    &body,
                    ir_program,
                    mir_program,
                    &call_unifier,
                    function_queue,
                    typedef_store,
                    &mut expr_id_map,
                    &mut pattern_id_map,
                );
                if let NamedFunctionKind::ExternClassImpl(class_name, ty) = &info.kind {
                    let mir_ty = process_type(ty, typedef_store, ir_program, mir_program);
                    MirFunctionInfo::ExternClassImpl(class_name.clone(), mir_ty, mir_expr_id)
                } else {
                    MirFunctionInfo::Normal(mir_expr_id)
                }
            } else {
                let constraints = call_unifier.get_constraints();
                for constraint in &constraints {
                    let module_name = {
                        let typedef_id = constraint.ty.get_typedef_id();
                        let typedef = ir_program.typedefs.get(&typedef_id);
                        match typedef {
                            TypeDef::Adt(adt) => adt.module.clone(),
                            TypeDef::Record(record) => record.module.clone(),
                        }
                    };
                    /*
                    println!(
                        "{} {} {}",
                        function.info,
                        constraint.class_id,
                        constraint.ty.get_resolved_type_string(ir_program)
                    );
                    */
                    if constraint.class_id == ir_program.get_ord_class_id() {
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_partialeq_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_partialord_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_eq_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_ord_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                    }
                    if constraint.class_id == ir_program.get_partialord_class_id() {
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_partialeq_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_partialord_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                    }
                    if constraint.class_id == ir_program.get_eq_class_id() {
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_partialeq_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                        let queue_item = FunctionQueueItem::ExternalCallImpl(
                            ir_program.get_eq_class_id(),
                            constraint.ty.clone(),
                            module_name.clone(),
                        );
                        function_queue.insert(queue_item, mir_program);
                    }
                    let queue_item = FunctionQueueItem::ExternalCallImpl(
                        constraint.class_id,
                        constraint.ty.clone(),
                        module_name,
                    );
                    function_queue.insert(queue_item, mir_program);
                }
                MirFunctionInfo::Extern(info.name.clone())
            };
            let mir_function = MirFunction {
                name: format!("{}_{}", info.name, mir_function_id.id),
                module: info.module.clone(),
                info: mir_function_info,
                arg_count: function.arg_count,
                function_type: mir_function_type,
                inline: function.inline,
            };
            mir_program
                .functions
                .add_item(mir_function_id, mir_function);
        }
        FunctionInfo::Lambda(info) => {
            preprocess_ir(info.body, ir_program);
            let mut expr_id_map = BTreeMap::new();
            let mut pattern_id_map = BTreeMap::new();
            let mir_body = process_expr(
                &info.body,
                ir_program,
                mir_program,
                &call_unifier,
                function_queue,
                typedef_store,
                &mut expr_id_map,
                &mut pattern_id_map,
            );
            let lambda_name = format!("{}{}", info, mir_function_id.id);
            let lambda_name = lambda_name.replace("/", "_");
            let lambda_name = lambda_name.replace(".", "_");
            let lambda_name = lambda_name.replace("#", "_");
            let mir_function = MirFunction {
                name: lambda_name,
                module: info.module.clone(),
                function_type: mir_function_type,
                arg_count: function.arg_count,
                info: MirFunctionInfo::Normal(mir_body),
                inline: function.inline,
            };
            mir_program
                .functions
                .add_item(mir_function_id, mir_function);
        }
        FunctionInfo::VariantConstructor(info) => {
            let adt = ir_program.typedefs.get(&info.type_id).get_adt();
            let variant = &adt.variants[info.index];
            let module = adt.module.clone();
            let result_ty = function_type.get_result_type(function.arg_count);
            let mir_typedef_id = typedef_store.add_type(result_ty, ir_program, mir_program);
            let name = format!(
                "{}_{}_ctor{}_{}",
                adt.name, variant.name, info.index, mir_function_id.id
            );
            let mir_function = MirFunction {
                name: name,
                module: module,
                function_type: mir_function_type,
                arg_count: function.arg_count,
                info: MirFunctionInfo::VariantConstructor(mir_typedef_id, info.index),
                inline: function.inline,
            };
            mir_program
                .functions
                .add_item(mir_function_id, mir_function);
        }
        FunctionInfo::RecordConstructor(info) => {
            let record = ir_program.typedefs.get(&info.type_id).get_record();
            let module = record.module.clone();
            let result_ty = function_type.get_result_type(function.arg_count);
            let mir_typedef_id = typedef_store.add_type(result_ty, ir_program, mir_program);
            let mir_function = MirFunction {
                name: format!("{}_ctor{}", record.name, mir_typedef_id.id),
                module: module,
                function_type: mir_function_type,
                arg_count: function.arg_count,
                info: MirFunctionInfo::RecordConstructor(mir_typedef_id),
                inline: function.inline,
            };
            mir_program
                .functions
                .add_item(mir_function_id, mir_function);
        }
    }
}
