use crate::siko::{
    location::{
        Location::Location,
        Report::{Report, ReportContext},
    },
    qualifiedname::QualifiedName,
};

pub enum MonomorphizerError {
    MissingMainFunction(QualifiedName),
    FunctionNotFound(QualifiedName),
    EffectHandlerMissing(QualifiedName, Location),
    EffectHandlerResolvesToSelf(QualifiedName, Location),
    ImplicitNotFound(QualifiedName, Location),
    UnusedEffectHandler {
        effect: QualifiedName,
        handler: QualifiedName,
        location: Location,
    },
}

impl MonomorphizerError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        self.report_only(ctx);
        std::process::exit(1);
    }

    pub fn report_only(&self, ctx: &ReportContext) {
        match self {
            MonomorphizerError::MissingMainFunction(name) => {
                let slogan = format!("No {} function found", ctx.yellow(&name.toString()));
                let r = Report::new(ctx, slogan, None);
                r.print();
            }
            MonomorphizerError::FunctionNotFound(name) => {
                let slogan = format!(
                    "Function {} not found during monomorphization, maybe std is missing?",
                    ctx.yellow(&name.toString())
                );
                let r = Report::new(ctx, slogan, None);
                r.print();
            }
            MonomorphizerError::EffectHandlerMissing(name, location) => {
                let slogan = format!(
                    "Effect method not present in current effect context: {}",
                    ctx.yellow(&name.toString())
                );
                let r = Report::new(ctx, slogan, Some(location.clone()));
                r.print();
            }
            MonomorphizerError::EffectHandlerResolvesToSelf(name, location) => {
                let slogan = format!("Effect handler {} resolves to itself", ctx.yellow(&name.toString()));
                let r = Report::new(ctx, slogan, Some(location.clone()));
                r.print();
                println!(
                    "   {} {}",
                    ctx.blue("help:"),
                    "Did you mean to reference a different function? The handler expression currently resolves back to the effect member itself."
                );
            }
            MonomorphizerError::ImplicitNotFound(name, location) => {
                let slogan = format!(
                    "Implicit variable not present in current implicit context: {}",
                    ctx.yellow(&name.toString())
                );
                let r = Report::new(ctx, slogan, Some(location.clone()));
                r.print();
            }
            MonomorphizerError::UnusedEffectHandler {
                effect,
                handler,
                location,
            } => {
                let slogan = format!(
                    "Unused effect handler {} for {}",
                    ctx.yellow(&handler.toString()),
                    ctx.yellow(&effect.toString())
                );
                let r = Report::new(ctx, slogan, Some(location.clone()));
                r.print();
            }
        }
    }
}
