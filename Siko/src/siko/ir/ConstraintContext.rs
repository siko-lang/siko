use super::Type::Type;

#[derive(Debug, Clone)]
pub struct ConstraintContext {
    pub typeParameters: Vec<String>,
    pub constraints: Vec<Type>,
}

impl ConstraintContext {
    pub fn new() -> ConstraintContext {
        ConstraintContext {
            typeParameters: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn add(&mut self, param: String) {
        self.typeParameters.push(param);
    }
}
