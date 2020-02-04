use crate::typedef_store::TypeDefStore;
use siko_ir::program::Program as IrProgram;
use siko_ir::types::Type as IrType;
use siko_mir::program::Program as MirProgram;
use siko_mir::types::Type as MirType;

pub fn process_type(
    ir_type: &IrType,
    typedef_store: &mut TypeDefStore,
    ir_program: &IrProgram,
    mir_program: &mut MirProgram,
) -> MirType {
    match ir_type {
        IrType::FixedTypeArg(..) => unreachable!(),
        IrType::Var(..) => unreachable!(),
        IrType::Function(from, to) => {
            let from = process_type(from, typedef_store, ir_program, mir_program);
            let to = process_type(to, typedef_store, ir_program, mir_program);
            let mir_type = MirType::Function(Box::new(from.clone()), Box::new(to.clone()));
            mir_type
        }
        IrType::Named(_, _, _) => {
            let mir_typedef_id = typedef_store.add_type(ir_type.clone(), ir_program, mir_program);
            MirType::Named(mir_typedef_id)
        }
        IrType::Tuple(items) => {
            let items: Vec<_> = items
                .iter()
                .map(|item| process_type(item, typedef_store, ir_program, mir_program))
                .collect();
            let (_, mir_typedef_id) = typedef_store.add_tuple(ir_type.clone(), items, mir_program);
            MirType::Named(mir_typedef_id)
        }
        IrType::Ref(item) => {
            let item = process_type(item, typedef_store, ir_program, mir_program);
            MirType::Ref(Box::new(item))
        }
    }
}

pub fn process_stored_type(ty: MirType, program: &mut MirProgram) -> MirType {
    if ty.is_function() {
        program.add_closure_type(&ty)
    } else {
        ty
    }
}
