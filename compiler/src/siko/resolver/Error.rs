use crate::siko::location::{
    Location::Location,
    Report::{Report, ReportContext},
};

pub enum ResolverError {
    UnknownValue(String, Location),
    NotAConstructor(String, Location),
    NotStructConstructor(String, Location),
    UnknownName(String, Location),
    Ambiguous(String, Location),
    RedundantPattern(Location),
    MissingPattern(Vec<String>, Location),
    BreakOutsideLoop(Location),
    ContinueOutsideLoop(Location),
    AssociatedTypeNotFound(String, String, Location),
    TraitNotFound(String, Location),
    InvalidInstanceMember(String, String, Location),
    MissingInstanceMembers(Vec<String>, String, Location),
    InvalidAssignment(Location),
    InvalidArgCount(String, i64, i64, Location),
    DynamicFunctionCallNotSupported(Location),
    ImmutableImplicit(String, Location),
}

impl ResolverError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        self.reportOnly(ctx);
        std::process::exit(1);
    }

    pub fn reportOnly(&self, ctx: &ReportContext) {
        match &self {
            ResolverError::UnknownValue(v, l) => {
                let slogan = format!("Unknown value {}", ctx.yellow(v));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::NotAConstructor(v, l) => {
                let slogan = format!("Not a constructor {}", ctx.yellow(v));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::NotStructConstructor(v, l) => {
                let slogan = format!("Not a struct constructor {}", ctx.yellow(v));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::UnknownName(v, l) => {
                let slogan = format!("Unknown name {}", ctx.yellow(v));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::RedundantPattern(l) => {
                let slogan = format!("Redundant pattern");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::MissingPattern(patterns, l) => {
                let pats: Vec<_> = patterns.iter().map(|p| ctx.yellow(p)).collect();
                let slogan = format!("Missing pattern(s): {}", pats.join(", "));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::BreakOutsideLoop(l) => {
                let slogan = format!("Break outside loop");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::ContinueOutsideLoop(l) => {
                let slogan = format!("Continue outside loop");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::AssociatedTypeNotFound(ty, traitName, l) => {
                let slogan = format!(
                    "Associated type {} not found in trait {}",
                    ctx.yellow(ty),
                    ctx.yellow(traitName)
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::TraitNotFound(traitName, l) => {
                let slogan = format!("Trait {} not found", ctx.yellow(traitName));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::InvalidInstanceMember(name, traitName, l) => {
                let slogan = format!(
                    "Member {} not found in trait {}",
                    ctx.yellow(name),
                    ctx.yellow(traitName)
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::MissingInstanceMembers(names, traitName, l) => {
                let names: Vec<_> = names.iter().map(|p| ctx.yellow(p)).collect();
                let slogan = format!(
                    "Missing instance member(s): {} for trait {}",
                    names.join(", "),
                    ctx.yellow(traitName)
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::Ambiguous(v, l) => {
                let slogan = format!("Ambiguous name {}", ctx.yellow(v));
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::InvalidAssignment(l) => {
                let slogan = format!("Invalid assignment target");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::InvalidArgCount(name, expected, actual, l) => {
                let slogan = format!(
                    "Invalid number of arguments for ctor {}. Expected {}, got {}",
                    ctx.yellow(name),
                    ctx.yellow(&expected.to_string()),
                    ctx.yellow(&actual.to_string())
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::DynamicFunctionCallNotSupported(l) => {
                let slogan = format!("Dynamic function calls are not supported");
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::ImmutableImplicit(name, location) => {
                let slogan = format!("Cannot modify immutable implicit variable {}", ctx.yellow(name));
                let r = Report::new(ctx, slogan, Some(location.clone()));
                r.print();
            }
        }
    }
}
