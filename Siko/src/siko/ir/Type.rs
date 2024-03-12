use crate::siko::qualifiedname::QualifiedName;

#[derive(Debug)]
pub enum Type {
    Named(QualifiedName, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Var(String),
    SelfType,
}
