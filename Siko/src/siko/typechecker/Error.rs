use crate::siko::location::{Location::Location, Report::Painter, Report::Report};

pub enum TypecheckerError {
    TypeMismatch(String, String, Location),
}

impl TypecheckerError {
    pub fn report(&self) -> ! {
        match &self {
            TypecheckerError::TypeMismatch(ty1, ty2, l) => {
                let slogan = format!("Type mismatch: {}, {}", ty1.yellow(), ty2.yellow());
                let r = Report::new(slogan, l.clone());
                r.print();
            }
        }
        std::process::exit(1);
    }
}
