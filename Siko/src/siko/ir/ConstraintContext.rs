use super::Type::Type;

#[derive(Debug, Clone)]
pub struct ConstraintContext {
    pub typeParameters: Vec<Type>,
    pub constraints: Vec<Type>,
}

impl ConstraintContext {
    pub fn new() -> ConstraintContext {
        ConstraintContext {
            typeParameters: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn add(&mut self, param: Type) {
        self.typeParameters.push(param);
    }
}
