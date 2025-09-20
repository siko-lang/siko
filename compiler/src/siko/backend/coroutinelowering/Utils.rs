use crate::siko::{
    hir::Type::Type,
    monomorphizer::Context::Context,
    qualifiedname::{builtins::getCoroutineCoResumeResultName, QualifiedName},
};

pub fn getLoweredCoroutineName(ty: &Type) -> QualifiedName {
    let (yielded, returnTy) = ty
        .clone()
        .unpackCoroutine()
        .expect("getLoweredCoroutineName: not a coroutine");
    QualifiedName::Coroutine(Box::new(yielded), Box::new(returnTy))
}

pub fn getLoweredCoroutineType(ty: &Type) -> Type {
    Type::Named(getLoweredCoroutineName(ty), Vec::new())
}

pub fn getMonomorphizedContext(ty: &Type) -> Context {
    let (yielded, returnTy) = ty
        .clone()
        .unpackCoroutine()
        .expect("getMonomorphizedContext: not a coroutine");
    let mut ctx = Context::new();
    ctx.args.push(yielded.clone());
    ctx.args.push(returnTy.clone());
    ctx
}

pub fn getResumeResultType(ty: &Type) -> Type {
    let ctx = getMonomorphizedContext(ty);
    let resultTy = Type::Named(getCoroutineCoResumeResultName().monomorphized(ctx), vec![]);
    resultTy
}

pub fn getResumeTupleType(ty: &Type) -> Type {
    let coroutineTy = getLoweredCoroutineType(ty);
    let resultTy = getResumeResultType(ty);
    Type::Tuple(vec![coroutineTy.clone(), resultTy.clone()])
}

pub fn getStateMachineEnumName(fName: &QualifiedName) -> QualifiedName {
    QualifiedName::CoroutineStateMachineEnum(Box::new(fName.clone()))
}
