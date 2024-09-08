use crate::siko::location::{Location::Location, Report::Painter, Report::Report};

pub enum ResolverError {
    UnknownValue(String, Location),
    UnknownName(String, Location),
    Ambiguous(String, Location),
    RedundantPattern(Location),
    MissingPattern(Location),
    BreakOutsideLoop(Location),
    ContinueOutsideLoop(Location),
    InvalidInstanceType(String, Location),
}

impl ResolverError {
    pub fn report(&self) -> ! {
        match &self {
            ResolverError::UnknownValue(v, l) => {
                let slogan = format!("Unknown value {}", v.yellow());
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::UnknownName(v, l) => {
                let slogan = format!("Unknown name {}", v.yellow());
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::RedundantPattern(l) => {
                let slogan = format!("Redundant pattern");
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::MissingPattern(l) => {
                let slogan = format!("Missing pattern");
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::BreakOutsideLoop(l) => {
                let slogan = format!("Break outside loop");
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::ContinueOutsideLoop(l) => {
                let slogan = format!("Continue outside loop");
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::InvalidInstanceType(ty, l) => {
                let slogan = format!("Invalid instance type {}", ty.yellow());
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
            ResolverError::Ambiguous(v, l) => {
                let slogan = format!("Ambiguous name {}", v.yellow());
                let r = Report::new(slogan, Some(l.clone()));
                r.print();
            }
        }
        std::process::exit(1);
    }
}
