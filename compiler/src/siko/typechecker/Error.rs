use crate::siko::location::{Location::Location, Report::Report, Report::ReportContext};

pub enum TypecheckerError {
    TypeMismatch(String, String, Location),
    FieldNotFound(String, String, Location),
    MethodNotFound(String, String, Location),
    MethodAmbiguous(String, Location),
    InstanceNotFound(String, String, Location),
    AmbiguousInstances(String, String, Location, Vec<Location>),
    TypeAnnotationNeeded(Location),
    ArgCountMismatch(u32, u32, Location),
    ImmutableAssign(Location),
    ImmutableImplicitHandler(Location),
    NotAPtr(String, Location),
    NoImplementationFound(String, Location),
    AmbiguousImplementations(String, String, Vec<String>, Location),
    YieldOutsideCoroutine(String, Location),
    NamedArgumentMismatch(String, String, String, Location),
}

impl TypecheckerError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        match &self {
            TypecheckerError::TypeMismatch(ty1, ty2, l) => {
                let slogan = format!("Type mismatch: {}, {}", ctx.yellow(ty1), ctx.yellow(ty2));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::FieldNotFound(name, ty, l) => {
                let slogan = format!("Field not found: {} on type {}", ctx.yellow(name), ctx.yellow(ty));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::MethodNotFound(name, ty, l) => {
                let slogan = format!("Method not found: {} on type {}", ctx.yellow(name), ctx.yellow(ty));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::MethodAmbiguous(name, l) => {
                let slogan = format!("Method ambiguous: {}", ctx.yellow(name));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::InstanceNotFound(traitName, params, l) => {
                let slogan = format!(
                    "Instance for {} not found with type(s): {}",
                    ctx.yellow(traitName),
                    ctx.yellow(params)
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::AmbiguousInstances(traitName, params, l, _) => {
                let slogan = format!(
                    "Instances for {} are ambiguous with type(s): {}",
                    ctx.yellow(traitName),
                    ctx.yellow(params)
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::TypeAnnotationNeeded(l) => {
                let slogan = format!("Type annotation needed");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::ArgCountMismatch(expected, found, l) => {
                let slogan = format!(
                    "Function argument count mismatch, expected: {}, found: {}",
                    ctx.yellow(&format!("{}", expected)),
                    ctx.yellow(&format!("{}", found))
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::ImmutableAssign(l) => {
                let slogan = format!("Value is not mutable, cannot assign");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::ImmutableImplicitHandler(l) => {
                let slogan = format!("Value is not mutable, cannot set as immutable handler");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::NotAPtr(ty, l) => {
                let slogan = format!("Value is not a pointer: {}", ctx.yellow(ty));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::NoImplementationFound(name, l) => {
                let slogan = format!("Instance for {} not found", ctx.yellow(name));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::AmbiguousImplementations(name, args, instances, location) => {
                let slogan = format!(
                    "Ambiguous implementations for {} with args: {}, instances: {}",
                    ctx.yellow(name),
                    ctx.yellow(args),
                    ctx.yellow(&instances.join(", "))
                );
                let r = Report::new(ctx, slogan, Some(location.clone()));
                r.print();
            }
            TypecheckerError::YieldOutsideCoroutine(fnName, l) => {
                let slogan = format!("Yield outside coroutine in function: {}", ctx.yellow(fnName));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            TypecheckerError::NamedArgumentMismatch(name, expectedName, fnName, l) => {
                let slogan = format!(
                    "Named argument {} does not match the parameter name {} in function: {}",
                    ctx.yellow(name),
                    ctx.yellow(expectedName),
                    ctx.yellow(fnName)
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
        }
        std::process::exit(1);
    }
}
