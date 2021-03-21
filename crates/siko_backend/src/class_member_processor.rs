use crate::function_queue::CallContext;
use crate::function_queue::FunctionQueue;
use crate::function_queue::FunctionQueueItem;
use crate::util::get_call_unifier;
use siko_ir::builder::Builder;
use siko_ir::class::ClassId;
use siko_ir::class::ClassMemberId;
use siko_ir::data::TypeDef as IrTypeDef;
use siko_ir::function::Function as IrFunction;
use siko_ir::function::FunctionId;
use siko_ir::function::FunctionInfo;
use siko_ir::function::NamedFunctionInfo;
use siko_ir::function::NamedFunctionKind;
use siko_ir::instance_resolver::ResolutionResult;
use siko_ir::program::Program as IrProgram;
use siko_ir::types::Type as IrType;
use siko_mir::function::FunctionId as MirFunctionId;
use siko_mir::program::Program as MirProgram;

#[derive(Debug)]
pub enum DerivedClass {
    Show,
    PartialEq,
    PartialOrd,
    Ord,
}

pub fn generate_auto_derived_instance_member(
    class_id: ClassId,
    ir_type: &IrType,
    ir_program: &mut IrProgram,
    derived_class: DerivedClass,
    class_member_id: ClassMemberId,
) -> FunctionId {
    let arg_count = match derived_class {
        DerivedClass::Show => 1,
        DerivedClass::PartialEq => 2,
        DerivedClass::PartialOrd => 2,
        DerivedClass::Ord => 2,
    };
    match ir_type {
        IrType::Named(_, typedef_id, _) => {
            let typedef = ir_program.typedefs.get(&typedef_id).clone();
            match typedef {
                IrTypeDef::Adt(adt) => {
                    let adt_type_info = ir_program
                        .adt_type_info_map
                        .get(&typedef_id)
                        .expect("Adt type info not found")
                        .clone();
                    let mut location = None;
                    for derived_class in &adt_type_info.derived_classes {
                        if derived_class.class_id == class_id {
                            location = Some(derived_class.location_id);
                            break;
                        }
                    }
                    let location = location.expect("Derive location not found");
                    let mut unifier = ir_program.get_unifier();
                    let r = unifier.unify(&adt_type_info.adt_type, &ir_type);
                    assert!(r.is_ok());
                    let function_id = ir_program.functions.get_id();
                    let (body, function_type) = match derived_class {
                        DerivedClass::Show => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_show_instance_member_for_adt(
                                location,
                                function_id,
                                &adt,
                                adt_type_info,
                            )
                        }
                        DerivedClass::PartialEq => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_partialeq_instance_member_for_adt(
                                location,
                                function_id,
                                &adt,
                                adt_type_info,
                                class_member_id,
                            )
                        }
                        DerivedClass::PartialOrd => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_partialord_instance_member_for_adt(
                                location,
                                function_id,
                                &adt,
                                adt_type_info,
                                class_member_id,
                            )
                        }
                        DerivedClass::Ord => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_ord_instance_member_for_adt(
                                location,
                                function_id,
                                &adt,
                                adt_type_info,
                                class_member_id,
                            )
                        }
                    };
                    let info = NamedFunctionInfo {
                        body: Some(body),
                        kind: NamedFunctionKind::Free,
                        location_id: location,
                        type_signature: None,
                        module: adt.module.clone(),
                        name: format!("{:?}", derived_class),
                    };
                    let function_info = FunctionInfo::NamedFunction(info);
                    let function = IrFunction {
                        id: function_id,
                        arg_count: arg_count,
                        arg_locations: vec![location],
                        info: function_info,
                        inline: true,
                    };
                    ir_program.function_types.insert(function_id, function_type);
                    ir_program.functions.add_item(function_id, function);
                    function_id
                }
                IrTypeDef::Record(record) => {
                    let record_type_info = ir_program
                        .record_type_info_map
                        .get(&typedef_id)
                        .expect("Record type info not found")
                        .clone();
                    let mut location = None;
                    for derived_class in &record_type_info.derived_classes {
                        if derived_class.class_id == class_id {
                            location = Some(derived_class.location_id);
                            break;
                        }
                    }
                    let location = location.expect("Derive location not found");
                    let mut unifier = ir_program.get_unifier();
                    let r = unifier.unify(&record_type_info.record_type, &ir_type);
                    assert!(r.is_ok());
                    let function_id = ir_program.functions.get_id();
                    let (body, function_type) = match derived_class {
                        DerivedClass::Show => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_show_instance_member_for_record(
                                location,
                                function_id,
                                &record,
                                record_type_info,
                            )
                        }
                        DerivedClass::PartialEq => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_partialeq_instance_member_for_record(
                                location,
                                function_id,
                                &record,
                                record_type_info,
                                class_member_id,
                            )
                        }
                        DerivedClass::PartialOrd => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_partialord_instance_member_for_record(
                                location,
                                function_id,
                                &record,
                                record_type_info,
                                class_member_id,
                            )
                        }
                        DerivedClass::Ord => {
                            let mut builder = Builder::new(ir_program);
                            builder.generate_ord_instance_member_for_record(
                                location,
                                function_id,
                                &record,
                                record_type_info,
                                class_member_id,
                            )
                        }
                    };
                    let info = NamedFunctionInfo {
                        body: Some(body),
                        kind: NamedFunctionKind::Free,
                        location_id: location,
                        type_signature: None,
                        module: record.module.clone(),
                        name: format!("{:?}", derived_class),
                    };
                    let function_info = FunctionInfo::NamedFunction(info);
                    let function = IrFunction {
                        id: function_id,
                        arg_count: arg_count,
                        arg_locations: vec![location],
                        info: function_info,
                        inline: true,
                    };
                    ir_program.function_types.insert(function_id, function_type);
                    ir_program.functions.add_item(function_id, function);
                    function_id
                }
            }
        }
        _ => unimplemented!(),
    }
}

pub fn process_class_member_call(
    arg_types: &Vec<IrType>,
    ir_program: &IrProgram,
    mir_program: &mut MirProgram,
    class_member_id: &ClassMemberId,
    expr_ty: IrType,
    function_queue: &mut FunctionQueue,
) -> MirFunctionId {
    for arg in arg_types {
        assert!(arg.is_concrete_type());
    }
    let member = ir_program.class_members.get(class_member_id);
    let (class_member_type, class_arg_ty) = ir_program
        .class_member_types
        .get(class_member_id)
        .expect("untyped class member");
    let call_unifier = get_call_unifier(
        arg_types,
        &class_member_type.remove_fixed_types(),
        &expr_ty,
        ir_program,
    );
    let class_arg = call_unifier.apply(&class_arg_ty.remove_fixed_types());
    assert!(class_arg.is_concrete_type());
    let context = CallContext::new(arg_types.clone(), expr_ty);
    match ir_program
        .instance_resolver
        .get(member.class_id, class_arg.clone())
    {
        ResolutionResult::AutoDerived => {
            if let Some(default_impl) = member.default_implementation {
                let queue_item = FunctionQueueItem::Normal(default_impl, context);
                let mir_function_id = function_queue.insert(queue_item, mir_program);
                mir_function_id
            } else {
                let queue_item = FunctionQueueItem::AutoDerive(
                    class_arg.clone(),
                    member.class_id,
                    *class_member_id,
                    context,
                );
                let mir_function_id = function_queue.insert(queue_item, mir_program);
                mir_function_id
            }
        }
        ResolutionResult::UserDefined(instance_id) => {
            let instance = ir_program.instances.get(&instance_id);
            let member_function_id =
                if let Some(instance_member) = instance.members.get(&member.name) {
                    instance_member.function_id
                } else {
                    member
                        .default_implementation
                        .expect("Default implementation not found")
                };
            let queue_item = FunctionQueueItem::Normal(member_function_id, context);
            let mir_function_id = function_queue.insert(queue_item, mir_program);
            mir_function_id
        }
    }
}
