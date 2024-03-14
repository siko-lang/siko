use crate::siko::location::{Location::Location, Report::Painter, Report::Report};

pub enum ResolverError {
    UnknownValue(String, Location),
}

impl ResolverError {
    pub fn report(&self) -> ! {
        match &self {
            ResolverError::UnknownValue(v, l) => {
                let slogan = format!("Unknown value {}", v.yellow());
                let r = Report::new(slogan, l.clone());
                r.print();
            }
        }
        std::process::exit(1);
    }
}
