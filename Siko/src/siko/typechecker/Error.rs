use crate::siko::location::{Location::Location, Report::Painter, Report::Report};

pub enum TypecheckerError {
    TypeMismatch(String, String, Location),
}

impl TypecheckerError {
    pub fn report(&self) -> ! {
        match &self {
            TypecheckerError::TypeMismatch(found, expected, l) => {
                let slogan = format!("Expected {}, found {}", expected.yellow(), found.yellow());
                let r = Report::new(slogan, l.clone());
                r.print();
            }
        }
        std::process::exit(1);
    }
}
