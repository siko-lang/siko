use crate::format_rewriter::FormatRewriter;
use siko_ir::expr::ExprId as IrExprId;
use siko_ir::program::Program as IrProgram;
use siko_ir::types::Type as IrType;
use siko_ir::unifier::Unifier;
use siko_ir::walker::walk_expr;

pub fn get_call_unifier(
    arg_types: &Vec<IrType>,
    func_ty: &IrType,
    expected_result_ty: &IrType,
    ir_program: &IrProgram,
) -> Unifier {
    /*
    for arg in arg_types {
        println!("arg {}", arg.get_resolved_type_string(ir_program));
        assert!(arg.is_concrete_type());
    }
    println!("{}", func_ty.get_resolved_type_string(ir_program));
    */
    let mut call_unifier = ir_program.get_unifier();
    let mut func_ty = func_ty.clone();
    for arg in arg_types {
        let mut func_arg_types = Vec::new();
        func_ty.get_args(&mut func_arg_types);
        let r = call_unifier.unify(&arg, &func_arg_types[0]);
        assert!(r.is_ok());
        func_ty.apply(&call_unifier);
        func_ty = func_ty.get_result_type(1);
    }
    /*
    println!(
        "{} {}",
        expected_result_ty.get_resolved_type_string(ir_program),
        func_ty.get_resolved_type_string(ir_program)
    );
    */
    let r = call_unifier.unify(&func_ty, expected_result_ty);
    assert!(r.is_ok());
    call_unifier
}

pub fn preprocess_ir(body: IrExprId, ir_program: &mut IrProgram) {
    let mut rewriter = FormatRewriter::new(ir_program);
    walk_expr(&body, &mut rewriter);
}
