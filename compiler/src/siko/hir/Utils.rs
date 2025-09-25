use crate::siko::{
    hir::{
        Function::Function, FunctionCallResolver::FunctionCallResolver, InstanceResolver::InstanceResolver,
        Program::Program, TypeVarAllocator::TypeVarAllocator, Unifier::Unifier,
    },
    location::Report::ReportContext,
    typechecker::ConstraintExpander::ConstraintExpander,
    util::Runner::Runner,
};

pub fn createResolvers<'a>(
    f: &'a Function,
    ctx: &'a ReportContext,
    program: &'a Program,
    runner: Runner,
) -> (InstanceResolver<'a>, FunctionCallResolver<'a>) {
    let instanceStore = program
        .instanceStores
        .get(&f.name.module())
        .expect("No impl store for module");
    let allocator = TypeVarAllocator::new();
    let expander = ConstraintExpander::new(program, allocator.clone(), f.constraintContext.clone());
    let knownConstraints = expander.expandKnownConstraints();
    let implResolver = InstanceResolver::new(allocator.clone(), instanceStore, program, knownConstraints.clone());
    let unifier = Unifier::withContext(ctx, runner.child("unifier"));
    let fnCallResolver = FunctionCallResolver::new(
        program,
        allocator.clone(),
        ctx,
        instanceStore,
        knownConstraints.clone(),
        unifier.clone(),
    );
    (implResolver, fnCallResolver)
}
