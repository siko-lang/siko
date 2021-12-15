use crate::class_member_processor::generate_auto_derived_instance_member;
use crate::class_member_processor::DerivedClass;
use crate::function_processor::process_function;
use crate::typedef_store::TypeDefStore;
use siko_ir::builder::Builder;
use siko_ir::class::ClassId;
use siko_ir::class::ClassMemberId;
use siko_ir::function::FunctionId as IrFunctionId;
use siko_ir::program::Program as IrProgram;
use siko_ir::types::Type as IrType;
use siko_mir::function::FunctionId as MirFunctionId;
use siko_mir::program::Program as MirProgram;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallContext {
    pub arg_types: Vec<IrType>,
    pub result_ty: IrType,
}

impl CallContext {
    pub fn new(arg_types: Vec<IrType>, result_ty: IrType) -> CallContext {
        CallContext {
            arg_types: arg_types,
            result_ty: result_ty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FunctionQueueItem {
    Normal(IrFunctionId, CallContext),
    AutoDerive(IrType, ClassId, ClassMemberId, CallContext),
    ExternalCallImpl(ClassId, IrType, String),
}

pub struct FunctionQueue {
    pending: Vec<(FunctionQueueItem, MirFunctionId)>,
    processed: BTreeMap<FunctionQueueItem, MirFunctionId>,
}

impl FunctionQueue {
    pub fn new() -> FunctionQueue {
        FunctionQueue {
            pending: Vec::new(),
            processed: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        item: FunctionQueueItem,
        mir_program: &mut MirProgram,
    ) -> MirFunctionId {
        let mut pending = false;
        let mir_function_id = self.processed.entry(item.clone()).or_insert_with(|| {
            pending = true;
            mir_program.functions.get_id()
        });
        if pending {
            self.pending.push((item, *mir_function_id));
        }
        *mir_function_id
    }

    pub fn process_items(
        &mut self,
        ir_program: &mut IrProgram,
        mir_program: &mut MirProgram,
        typedef_store: &mut TypeDefStore,
    ) {
        while !self.pending.is_empty() {
            if let Some((item, mir_function_id)) = self.pending.pop() {
                match item {
                    FunctionQueueItem::Normal(function_id, call_context) => {
                        process_function(
                            &function_id,
                            mir_function_id,
                            ir_program,
                            mir_program,
                            call_context.arg_types,
                            call_context.result_ty,
                            self,
                            typedef_store,
                        );
                    }
                    FunctionQueueItem::AutoDerive(
                        ir_type,
                        class_id,
                        class_member_id,
                        call_context,
                    ) => {
                        let class = ir_program.classes.get(&class_id);
                        let function_id = match (class.module.as_ref(), class.name.as_ref()) {
                            ("Std.Ops", "Show") => generate_auto_derived_instance_member(
                                class_id,
                                &ir_type,
                                ir_program,
                                DerivedClass::Show,
                                class_member_id,
                            ),
                            ("Std.Ops", "PartialEq") => generate_auto_derived_instance_member(
                                class_id,
                                &ir_type,
                                ir_program,
                                DerivedClass::PartialEq,
                                class_member_id,
                            ),
                            ("Std.Ops", "PartialOrd") => generate_auto_derived_instance_member(
                                class_id,
                                &ir_type,
                                ir_program,
                                DerivedClass::PartialOrd,
                                class_member_id,
                            ),
                            ("Std.Ops", "Ord") => generate_auto_derived_instance_member(
                                class_id,
                                &ir_type,
                                ir_program,
                                DerivedClass::Ord,
                                class_member_id,
                            ),
                            _ => panic!(
                                "Auto derive of {}/{} is not implemented",
                                class.module, class.name
                            ),
                        };
                        let queue_item = FunctionQueueItem::Normal(function_id, call_context);
                        self.pending.push((queue_item.clone(), mir_function_id));
                        self.processed.insert(queue_item, mir_function_id);
                    }
                    FunctionQueueItem::ExternalCallImpl(class_id, ir_type, module) => {
                        let class = ir_program.classes.get(&class_id);
                        let location_id = class.location_id;
                        if class_id == ir_program.get_show_class_id() {
                            let string_ty = ir_program.get_string_type();
                            let class_member_id = ir_program.get_show_member_id();
                            let mut builder = Builder::new(ir_program);
                            let func_id = builder.generate_extern_class_impl(
                                location_id,
                                format!("ExternClassImpl{}", class_id),
                                module.clone(),
                                1,
                                ir_type.clone(),
                                string_ty,
                                class_member_id,
                            );
                            let string_ty = ir_program.get_string_type();
                            let context = CallContext::new(vec![ir_type.clone()], string_ty);
                            let queue_item = FunctionQueueItem::Normal(func_id, context);
                            self.insert(queue_item, mir_program);
                        } else if class_id == ir_program.get_ord_class_id() {
                            let ordering_ty = ir_program.get_ordering_type();
                            let class_member_id = ir_program.get_cmp_member_id();
                            let mut builder = Builder::new(ir_program);
                            let func_id = builder.generate_extern_class_impl(
                                location_id,
                                format!("ExternClassImpl{}", class_id),
                                module.clone(),
                                2,
                                ir_type.clone(),
                                ordering_ty.clone(),
                                class_member_id,
                            );
                            let context = CallContext::new(
                                vec![ir_type.clone(), ir_type.clone()],
                                ordering_ty,
                            );
                            let queue_item = FunctionQueueItem::Normal(func_id, context);
                            self.insert(queue_item, mir_program);
                        } else if class_id == ir_program.get_partialeq_class_id() {
                            let bool_ty = ir_program.get_bool_type();
                            let class_member_id = ir_program.get_opeq_member_id();
                            let mut builder = Builder::new(ir_program);
                            let func_id = builder.generate_extern_class_impl(
                                location_id,
                                format!("ExternClassImpl{}", class_id),
                                module.clone(),
                                2,
                                ir_type.clone(),
                                bool_ty.clone(),
                                class_member_id,
                            );
                            let context =
                                CallContext::new(vec![ir_type.clone(), ir_type.clone()], bool_ty);
                            let queue_item = FunctionQueueItem::Normal(func_id, context);
                            self.insert(queue_item, mir_program);
                        } else if class_id == ir_program.get_partialord_class_id() {
                            let optional_ordering_ty =
                                ir_program.get_option_type(ir_program.get_ordering_type());
                            let class_member_id = ir_program.get_partialcmp_member_id();
                            let mut builder = Builder::new(ir_program);
                            let func_id = builder.generate_extern_class_impl(
                                location_id,
                                format!("ExternClassImpl{}", class_id),
                                module.clone(),
                                2,
                                ir_type.clone(),
                                optional_ordering_ty.clone(),
                                class_member_id,
                            );
                            let context = CallContext::new(
                                vec![ir_type.clone(), ir_type.clone()],
                                optional_ordering_ty,
                            );
                            let queue_item = FunctionQueueItem::Normal(func_id, context);
                            self.insert(queue_item, mir_program);
                        } else if class_id == ir_program.get_eq_class_id() {
                            let mut builder = Builder::new(ir_program);
                            let func_id = builder.generate_extern_eq_impl(
                                location_id,
                                format!("ExternClassImpl{}", class_id),
                                module.clone(),
                                ir_type.clone(),
                            );
                            let context = CallContext::new(vec![], IrType::Tuple(Vec::new()));
                            let queue_item = FunctionQueueItem::Normal(func_id, context);
                            self.insert(queue_item, mir_program);
                        } else {
                            println!(
                                "Unimplemented extern class call {} {} {}",
                                class_id,
                                class.name,
                                ir_type.get_resolved_type_string(ir_program)
                            );
                        }
                    }
                }
            }
        }
    }
}
