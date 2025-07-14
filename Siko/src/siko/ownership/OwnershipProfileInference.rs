use crate::siko::{
    hir::{Function::FunctionKind, Program::Program},
    ownership::{DataOwnershipVar::DataOwnershipVarInference, FunctionGroups},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::DependencyGroup,
};

struct OwnershipProfile {}

fn processStructCtorFunction() -> OwnershipProfile {
    OwnershipProfile {}
}

fn processFunctionGroup(group: &DependencyGroup<QualifiedName>, program: &Program) {
    println!("Processing function group: {:?}", group);
    if group.items.len() == 1 {
        let item = &group.items[0];
        let function = program.functions.get(item).unwrap();
        match &function.kind {
            FunctionKind::UserDefined => {
                println!("Processing user-defined function: {:?}", item);
            }
            FunctionKind::VariantCtor(_) => {
                println!("Processing variant constructor for {:?}", item);
            }
            FunctionKind::StructCtor => {
                println!("Processing struct constructor for {:?}", item);
            }
            FunctionKind::Extern => {
                println!("Processing external function: {:?}", item);
            }
            FunctionKind::TraitMemberDecl(qualified_name) => {
                panic!("Trait member declaration not expected here: {:?}", qualified_name);
            }
            FunctionKind::TraitMemberDefinition(qualified_name) => {
                panic!("Trait member definition not expected here: {:?}", qualified_name);
            }
        }
    } else {
    }
    // Here you can add logic to process each function group
}

pub fn ownershipInference(program: Program) {
    let data_lifetime_inferer = DataOwnershipVarInference::new(program);
    let program = data_lifetime_inferer.process();
    let groups = FunctionGroups::createFunctionGroups(&program.functions);
    for group in groups {
        processFunctionGroup(&group, &program);
    }
}
