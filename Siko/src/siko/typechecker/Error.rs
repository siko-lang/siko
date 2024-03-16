use crate::siko::location::{Location::Location, Report::Painter, Report::Report};

pub enum TypecheckerError {
    TypeMismatch(String, String, Location),
    FieldNotFound(String, Location),
    TypeAnnotationNeeded(Location),
    ArgCountMismatch(u32, u32, Location),
}

impl TypecheckerError {
    pub fn report(&self) -> ! {
        match &self {
            TypecheckerError::TypeMismatch(ty1, ty2, l) => {
                let slogan = format!("Type mismatch: {}, {}", ty1.yellow(), ty2.yellow());
                let r = Report::new(slogan, l.clone());
                r.print();
            }
            TypecheckerError::FieldNotFound(name, l) => {
                let slogan = format!("Field not found: {}", name.yellow());
                let r = Report::new(slogan, l.clone());
                r.print();
            }
            TypecheckerError::TypeAnnotationNeeded(l) => {
                let slogan = format!("Type annotation needed");
                let r = Report::new(slogan, l.clone());
                r.print();
            }
            TypecheckerError::ArgCountMismatch(expected, found, l) => {
                let slogan = format!(
                    "Function argument count mismatch, expected: {}, found: {}",
                    format!("{}", expected).yellow(),
                    format!("{}", found).yellow()
                );
                let r = Report::new(slogan, l.clone());
                r.print();
            }
        }
        std::process::exit(1);
    }
}
